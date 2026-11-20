use crate::{Agent, Error, ErrorKind, Result, Session, schema};
use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{future::Future, marker::PhantomData, pin::Pin, time::Duration};

/// Owned subprocess configuration for an ACP agent.
///
/// The executable must speak ACP over standard input/output and support Thinkwell's
/// HTTP MCP path. Arguments are passed directly without a shell. The child inherits
/// the environment; [env](Self::env) adds overrides. Installation and authentication
/// are the application's responsibility.
///
/// Creating this value launches nothing. [Agent::connect] validates the bounds
/// and establishes the connection. Custom presets can implement `Into<AgentConfig>`
/// and automatically gain the [Connect] extension method.
///
/// # Example
///
/// ~~~rust
/// use std::time::Duration;
/// use thinkwell::AgentConfig;
/// let mut config = AgentConfig::new("my-acp-agent")
///     .args(["--transport", "stdio"])
///     .env("LOG_LEVEL", "warn");
/// config.setup_timeout = Duration::from_secs(45);
/// config.shutdown_timeout = Duration::from_secs(15);
/// ~~~
#[derive(Clone, Debug)]
pub struct AgentConfig {
    pub(crate) command: String,
    pub(crate) args: Vec<String>,
    pub(crate) env: Vec<(String, String)>,
    /// Deadline for initialization and, separately, opening/configuring a session.
    ///
    /// Defaults to 30 seconds. Must be nonzero and fit the host clock.
    /// This does not impose a lifetime limit on an idle connected agent.
    pub setup_timeout: Duration,
    /// Deadline for awaited shutdown confirmation and HTTP cleanup.
    ///
    /// Defaults to 12 seconds. Must be nonzero and fit the host clock. A deadline
    /// error means cleanup was not confirmed in time; destruction still signals
    /// the existing cleanup driver.
    pub shutdown_timeout: Duration,
}
impl AgentConfig {
    /// Configure an executable with no arguments or environment overrides.
    ///
    /// Setup and shutdown default to 30 and 12 seconds. An empty command is rejected
    /// when connecting. See [AgentConfig] for a configuration example.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: vec![],
            env: vec![],
            setup_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(12),
        }
    }
    /// Append command-line arguments in order.
    ///
    /// Repeated calls append rather than replace. Each string is one argument; shell
    /// quoting, variable expansion, and command substitution are not performed.
    ///
    /// ~~~rust
    /// use thinkwell::AgentConfig;
    /// let config = AgentConfig::new("my-acp-agent")
    ///     .args(["--transport", "stdio"])
    ///     .args(["--log-level", "warn"]);
    /// ~~~
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }
    /// Add an environment override for the child process.
    ///
    /// Other variables are inherited. Later overrides of the same name take precedence.
    /// This does not change the parent environment. Configurations may contain sensitive
    /// values, so avoid logging them indiscriminately.
    pub fn env(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((name.into(), value.into()));
        self
    }
}
/// Launch preset for the Claude ACP adapter.
///
/// The default command is
/// `npx --offline --yes @zed-industries/claude-agent-acp@0.23.1`.
/// Node/npx, the cached adapter, and authentication must already be available.
/// Thinkwell does not install or authenticate it. Model selection defaults to the
/// agent's choice; use [SessionBuilder::model](crate::SessionBuilder::model) to override it.
///
/// [Claude::connect] is the default-preset shorthand. Import [Connect] to call
/// `.connect()` on an instance.
///
/// # Example
///
/// ~~~no_run
/// use thinkwell::{Claude, Connect};
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// agent.shutdown().await?;
/// let preset = Claude::default();
/// let another = preset.connect().await?;
/// another.shutdown().await?;
/// # Ok(())
/// # }
/// ~~~
#[derive(Clone, Debug)]
pub struct Claude {
    config: AgentConfig,
}
impl Default for Claude {
    fn default() -> Self {
        Self {
            config: AgentConfig::new("npx").args([
                "--offline",
                "--yes",
                "@zed-industries/claude-agent-acp@0.23.1",
            ]),
        }
    }
}
impl Claude {
    /// Wrap an explicit launch configuration as a Claude preset.
    ///
    /// Replaces the whole command, including arguments; it does not merge with the
    /// offline npx defaults.
    ///
    /// ~~~no_run
    /// use thinkwell::{AgentConfig, Claude, Connect};
    /// # async fn example() -> thinkwell::Result<()> {
    /// let preset = Claude::configured(AgentConfig::new("claude-agent-acp"));
    /// let agent = preset.connect().await?;
    /// agent.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ~~~
    pub fn configured(config: AgentConfig) -> Self {
        Self { config }
    }
    /// Connect using the default Claude preset.
    ///
    /// Equivalent to `Agent::connect(Claude::default()).await`. See [Claude] for
    /// prerequisites and examples, and [Agent::connect] for runtime and error behavior.
    pub async fn connect() -> Result<Agent> {
        Agent::connect(Self::default()).await
    }
}
impl From<Claude> for AgentConfig {
    fn from(value: Claude) -> Self {
        value.config
    }
}
/// Connection convenience for any value convertible to [AgentConfig].
///
/// The blanket implementation lets applications add presets without extending a
/// provider enum. Import this trait to use the instance method; the inherent
/// [Claude::connect] shorthand needs no trait import.
///
/// # Example: an application-owned preset
///
/// ~~~no_run
/// use thinkwell::{AgentConfig, Connect};
/// struct LocalAgent;
/// impl From<LocalAgent> for AgentConfig {
///     fn from(_: LocalAgent) -> Self {
///         AgentConfig::new("my-acp-agent").args(["--transport", "stdio"])
///     }
/// }
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = LocalAgent.connect().await?;
/// agent.shutdown().await?;
/// # Ok(())
/// # }
/// ~~~
pub trait Connect: Into<AgentConfig> + Sized {
    /// Consume this preset and connect through [Agent::connect].
    ///
    /// The future launches when polled and inherits the same runtime requirements,
    /// errors, and resource-cleanup behavior.
    fn connect(self) -> impl Future<Output = Result<Agent>> {
        Agent::connect(self)
    }
}
impl<T: Into<AgentConfig>> Connect for T {}

/// Per-execution time and call budgets, set with [Plan::options].
///
/// Defaults allow a 120-second turn, a 30-second callback, 32 application calls,
/// and three invalid result submissions. These are execution bounds, not token
/// limits. Fields must be nonzero and time bounds must fit the host clock.
/// Invalid options fail before prompting.
///
/// # Example
///
/// ~~~rust
/// use std::time::Duration;
/// use thinkwell::{Plan, RunOptions};
/// let options = RunOptions {
///     turn_timeout: Duration::from_secs(60),
///     callback_timeout: Duration::from_secs(10),
///     max_tool_calls: 8,
///     max_invalid_results: 2, // One correction after the first invalid attempt.
/// };
/// let plan = Plan::<String>::new().text("Answer briefly.").options(options);
/// ~~~
#[derive(Clone, Debug)]
pub struct RunOptions {
    /// Maximum duration of local turn processing, including model work, callbacks,
    /// and waiting for terminal completion. Expiry closes the session and connection.
    pub turn_timeout: Duration,
    /// Maximum duration of one local invocation, including validation.
    ///
    /// Expiry drops its future and returns a tool error to the agent, which may
    /// continue within the turn budget. Earlier mutations are not rolled back.
    pub callback_timeout: Duration,
    /// Maximum matching application-tool requests dispatched per turn.
    ///
    /// Invalid application input and callback errors count. The next request beyond
    /// the limit fails the turn. Stale-identifier rejections and result submissions
    /// do not consume this budget.
    pub max_tool_calls: usize,
    /// Number of invalid result submissions that exhausts the turn budget.
    ///
    /// The default of three allows two corrections. Missing values, schema violations,
    /// and deserialization failures count; stale identifiers and duplicates after
    /// a valid candidate do not.
    pub max_invalid_results: usize,
}
impl Default for RunOptions {
    fn default() -> Self {
        Self {
            turn_timeout: Duration::from_secs(120),
            callback_timeout: Duration::from_secs(30),
            max_tool_calls: 32,
            max_invalid_results: 3,
        }
    }
}
impl RunOptions {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.turn_timeout.is_zero()
            || self.callback_timeout.is_zero()
            || std::time::Instant::now()
                .checked_add(self.turn_timeout)
                .is_none()
            || std::time::Instant::now()
                .checked_add(self.callback_timeout)
                .is_none()
            || self.max_tool_calls == 0
            || self.max_invalid_results == 0
        {
            Err(Error::new(
                ErrorKind::Configuration,
                "run bounds must be nonzero",
            ))
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Descriptor {
    pub name: String,
    pub description: String,
    pub input: Value,
    pub output: Value,
}
/// A named contract with input type `I` and output type `O`.
///
/// Declare tools with [SessionBuilder::tool](crate::SessionBuilder::tool) before
/// opening a session, then supply each turn's implementation with [Plan::bind].
/// Cloning a descriptor does not capture application state or retain a callback.
/// Bindings match the declared name, description, and generated schemas.
///
/// Inputs use Serde deserialization; outputs use serialization. Both derive
/// `schemars::JsonSchema` and are validated at the boundary. See the crate's
/// [schema support](crate#schema-support). Protocol envelopes do not change
/// the callback's Rust input or output types.
///
/// # Example
///
/// ~~~rust
/// use schemars::JsonSchema;
/// use serde::{Deserialize, Serialize};
/// use thinkwell::Tool;
/// #[derive(Deserialize, JsonSchema)]
/// struct Add { value: i64 }
/// #[derive(Serialize, JsonSchema)]
/// struct Total { total: i64 }
/// let add = Tool::<Add, Total>::new("add", "Add to the running total");
/// let same_contract = add.clone();
/// ~~~
///
/// See [Plan] for binding a tool to borrowed application state.
pub struct Tool<I, O> {
    pub(crate) descriptor: Result<Descriptor>,
    types: PhantomData<fn(I) -> O>,
}
impl<I, O> Clone for Tool<I, O> {
    fn clone(&self) -> Self {
        Self {
            descriptor: self.descriptor.clone(),
            types: PhantomData,
        }
    }
}
impl<I: DeserializeOwned + JsonSchema, O: Serialize + JsonSchema> Tool<I, O> {
    /// Declare the tool name, agent-facing description, and derived contracts.
    ///
    /// Names must be nonempty ASCII letters, digits, underscores, or hyphens.
    /// `return_result` is reserved. Invalid names or unsupported schemas are retained
    /// and reported when opening a session or executing a plan, before prompting;
    /// construction itself does not return Result. See [Tool] for an example.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let name = name.into();
        let descriptor = (|| {
            if name.is_empty()
                || name == "return_result"
                || !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Err(Error::new(
                    ErrorKind::Binding,
                    "invalid or reserved tool name",
                ));
            }
            Ok(Descriptor {
                name,
                description: description.into(),
                input: schema::contract::<I>()?,
                output: schema::contract::<O>()?,
            })
        })();
        Self {
            descriptor,
            types: PhantomData,
        }
    }
}
pub(crate) trait Callback {
    fn call(&mut self, input: Value) -> Pin<Box<dyn Future<Output = Result<Value>> + '_>>;
}
struct TypedCallback<I, O, F> {
    f: F,
    descriptor: Descriptor,
    types: PhantomData<fn(I) -> O>,
}
impl<I: DeserializeOwned + 'static, O: Serialize + 'static, F: AsyncFnMut(I) -> Result<O>> Callback
    for TypedCallback<I, O, F>
{
    fn call(&mut self, input: Value) -> Pin<Box<dyn Future<Output = Result<Value>> + '_>> {
        Box::pin(async move {
            let input = schema::decode::<I>(&self.descriptor.input, input)?;
            let output = (self.f)(input).await?;
            let value = serde_json::to_value(output)
                .map_err(|e| Error::new(ErrorKind::InvalidValue, e.to_string()))?;
            schema::validate(&self.descriptor.output, &value)?;
            Ok(value)
        })
    }
}
/// Marker for a [Plan] without callbacks, which can be cloned.
///
/// This is the default plan state. [Plan::bind] changes it to [Bound].
/// Applications normally rely on inference rather than naming the markers.
pub struct Unbound;
/// Marker for a [Plan] owning callback bindings.
///
/// Bound plans are not Clone because their closures may exclusively borrow state.
/// Consuming or dropping the plan releases its captures.
pub struct Bound;
pub(crate) struct Binding<'a> {
    pub descriptor: Descriptor,
    pub callback: Box<dyn Callback + 'a>,
}
/// A prompt configuration whose successful final result has type `T`.
///
/// Build a reusable unbound plan with [new](Self::new), [text](Self::text), and
/// [options](Self::options). [bind](Self::bind) consumes it and produces a
/// non-cloneable plan owning callbacks. Execute with [run_in](Self::run_in)
/// against a persistent [Session].
///
/// Callbacks may borrow local state and need not be Send. Each execution exclusively
/// borrows its session. After consuming it with `.await`, application code can access
/// the captured state again. Later turns may change result types without replacing
/// conversation history.
///
/// # Example: borrowed state and differently typed turns
///
/// ~~~no_run
/// use schemars::JsonSchema;
/// use serde::{Deserialize, Serialize};
/// use thinkwell::{Claude, Plan, Tool};
/// #[derive(Deserialize, JsonSchema)]
/// struct Add { value: i64 }
/// #[derive(Serialize, JsonSchema)]
/// struct Total { total: i64 }
/// #[derive(Deserialize, JsonSchema)]
/// struct Summary { total: i64 }
///
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// let add = Tool::<Add, Total>::new("add", "Add to the accumulator");
/// let mut session = agent.session().tool(&add).open().await?;
/// let mut total = 0;
/// let base = Plan::<Summary>::new().text("Add 3 and report the total.");
/// let summary = base.clone()
///     .bind(&add, async |input: Add| {
///         total += input.value;
///         Ok(Total { total })
///     })
///     .run_in(&mut session).await?;
///
/// total *= 2; // The first callback's borrow has ended.
/// let explanation: String = Plan::new()
///     .text(format!("Add 4 and explain the change from {}.", summary.total))
///     .bind(&add, async |input: Add| {
///         total += input.value;
///         Ok(Total { total })
///     })
///     .run_in(&mut session).await?;
/// println!("{explanation}: {total}");
/// # Ok(())
/// # }
/// ~~~
///
/// Every declared tool requires a binding on every turn, even if the agent need not
/// call it. Ordinary Drop handles teardown; awaited connection shutdown is optional.
pub struct Plan<'a, T, S = Unbound> {
    pub(crate) text: String,
    pub(crate) bindings: Vec<Binding<'a>>,
    pub(crate) error: Option<Error>,
    pub(crate) options: RunOptions,
    types: PhantomData<fn() -> (T, S)>,
}
impl<T> Plan<'_, T> {
    /// Create an empty unbound plan with default [RunOptions].
    ///
    /// The result schema is checked during execution preflight. Constructing or
    /// cloning a plan does not launch an agent or send a prompt.
    ///
    /// ~~~rust
    /// use thinkwell::Plan;
    /// let base = Plan::<String>::new().text("Answer concisely.");
    /// let first = base.clone().text("What is ACP?");
    /// let second = base.text("What is MCP?");
    /// ~~~
    pub fn new() -> Self {
        Self {
            text: String::new(),
            bindings: vec![],
            error: None,
            options: RunOptions::default(),
            types: PhantomData,
        }
    }
}
impl<T> Default for Plan<'_, T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Clone for Plan<'_, T> {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            bindings: vec![],
            error: self.error.clone(),
            options: self.options.clone(),
            types: PhantomData,
        }
    }
}
impl<'a, T, S> Plan<'a, T, S> {
    /// Append literal prompt text and a newline.
    ///
    /// Calls concatenate in order. No escaping, interpolation, or untrusted-content
    /// boundary is added. Thinkwell appends its protocol instructions during execution.
    pub fn text(mut self, text: impl AsRef<str>) -> Self {
        self.text.push_str(text.as_ref());
        self.text.push('\n');
        self
    }
    /// Replace execution budgets; see [RunOptions] for an example.
    ///
    /// Validation occurs before prompting. Cloning an unbound plan preserves options.
    pub fn options(mut self, options: RunOptions) -> Self {
        self.options = options;
        self
    }
    /// Bind a declared tool to an async callback for this turn.
    ///
    /// The callback accepts `I` and returns `thinkwell::Result<O>`. It may borrow local
    /// state and hold non-Send values across await; owned I/O tasks never hold its
    /// captures. See [Plan] for a complete example.
    ///
    /// Inputs are validated/deserialized before the callback; outputs are serialized
    /// and validated afterward. [Error::tool] reports an application failure to the
    /// agent, which may handle it. Earlier callback effects are not undone.
    ///
    /// Duplicate, missing, mismatched, or invalid bindings fail execution preflight.
    /// This method retains errors instead of returning Result and changes state to
    /// [Bound]. Captures live until the plan/execution is consumed or dropped: even a
    /// completed future held in pinned storage can retain them.
    pub fn bind<I, O, F>(mut self, tool: &Tool<I, O>, f: F) -> Plan<'a, T, Bound>
    where
        I: DeserializeOwned + 'static,
        O: Serialize + 'static,
        F: AsyncFnMut(I) -> Result<O> + 'a,
    {
        match &tool.descriptor {
            Ok(d) => self.bindings.push(Binding {
                descriptor: d.clone(),
                callback: Box::new(TypedCallback {
                    f,
                    descriptor: d.clone(),
                    types: PhantomData,
                }),
            }),
            Err(e) => self.error = Some(e.clone()),
        }
        Plan {
            text: self.text,
            bindings: self.bindings,
            error: self.error,
            options: self.options,
            types: PhantomData,
        }
    }
    /// Execute a turn in an existing conversation and return its typed result.
    ///
    /// Preflight checks options, result schema, and exact tool bindings before sending
    /// a prompt. Success requires a validated/deserialized result candidate, normal
    /// terminal completion, and settled local callback work. A candidate alone does
    /// not finish the future; later submissions cannot replace an accepted candidate.
    ///
    /// # Errors and cancellation
    ///
    /// Preflight failures leave the session reusable. Any failed started turn closes
    /// the session and connection, including exhaustion, deadline, missing result,
    /// abnormal stop, or transport loss. Inspect [Error::kind] and [ErrorKind]
    /// rather than parsing messages.
    ///
    /// The future drives local callbacks and must be polled. Dropping it unpolled sends
    /// no prompt. Dropping a started execution releases callback access and closes the
    /// connection; it cannot roll back effects or await remote acknowledgment.
    ///
    /// # Example: a conversation without application tools
    ///
    /// ~~~no_run
    /// use thinkwell::{Plan, Session};
    /// async fn ask(session: &mut Session<'_>) -> thinkwell::Result<()> {
    ///     let count = Plan::<i64>::new().text("How many days are in a week?")
    ///         .run_in(session).await?;
    ///     let explanation = Plan::<String>::new()
    ///         .text(format!("Explain that answer of {count} in one sentence."))
    ///         .run_in(session).await?;
    ///     println!("{explanation}");
    ///     Ok(())
    /// }
    /// ~~~
    ///
    /// This helper expects an empty tool catalog. See [Plan] for borrowed tools.
    ///
    /// # Compile-time ownership
    ///
    /// Overlapping executions cannot borrow the same session:
    ///
    /// ```compile_fail,E0499
    /// use thinkwell::{Plan,Session};
    /// async fn overlap(s:&mut Session<'_>) {
    ///     let a=Plan::<String>::new().run_in(s);
    ///     let b=Plan::<String>::new().run_in(s);
    ///     let _=a.await; let _=b.await;
    /// }
    /// ```
    ///
    /// Captured state also remains exclusively borrowed:
    ///
    /// ```compile_fail,E0503
    /// use thinkwell::{Plan,Session,Tool};
    /// async fn capture(session: &mut Session<'_>, tool: &Tool<String,String>) {
    ///     let mut count=0;
    ///     let execution=Plan::<String>::new().bind(tool,async |value:String| {
    ///         count+=1; Ok(value)
    ///     }).run_in(session);
    ///     count+=1;
    ///     let _=execution.await;
    /// }
    /// ```
    pub async fn run_in(self, session: &mut Session<'_>) -> Result<T>
    where
        T: DeserializeOwned + JsonSchema,
    {
        session.run(self).await
    }
}
