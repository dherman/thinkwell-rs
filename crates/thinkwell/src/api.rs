use crate::{Agent, Error, ErrorKind, Result, Session, schema};
use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{future::Future, marker::PhantomData, pin::Pin, time::Duration};

/// The command Thinkwell should run to start an agent.
///
/// Use this to connect to an agent other than the [built-in presets](crate::agent),
/// or choose its command-line arguments. Creating a configuration does not
/// start anything; pass it to [Agent::connect] when you are ready.
///
/// ~~~no_run
/// use thinkwell::{Agent, AgentConfig};
///
/// # async fn example() -> thinkwell::Result<()> {
/// let config = AgentConfig::new("my-acp-agent")
///     .args(["--transport", "stdio"])
///     .env("LOG_LEVEL", "warn");
/// let agent = Agent::connect(config).await?;
/// # Ok(())
/// # }
/// ~~~
///
/// The command must run an agent that speaks ACP, the protocol Thinkwell uses to
/// talk to agents. This version also requires HTTP MCP support so Thinkwell can
/// offer tools to the agent. Install and authenticate the agent before connecting.
///
/// Arguments are passed directly, without a shell. The child inherits your
/// application's environment, with any overrides added by [env](Self::env).
/// See [Connect] if you want to package a configuration as your own reusable preset.
#[derive(Clone, Debug)]
pub struct AgentConfig {
    pub(crate) command: String,
    pub(crate) args: Vec<String>,
    pub(crate) env: Vec<(String, String)>,
    /// How long to wait for connection setup, or for opening a conversation.
    ///
    /// Defaults to 30 seconds. Each setup operation gets its own timeout; this does
    /// not limit how long you can keep an idle agent connected. Must be nonzero
    /// and fit the host clock.
    pub setup_timeout: Duration,
    /// How long to wait when confirming cleanup with [Agent::shutdown].
    ///
    /// Defaults to 12 seconds and also bounds HTTP cleanup. A timeout means cleanup
    /// was not confirmed in time. Must be nonzero and fit the host clock.
    pub shutdown_timeout: Duration,
}
impl AgentConfig {
    /// Choose the executable to run.
    ///
    /// Starts with no arguments or environment overrides. Use [args](Self::args) and
    /// [env](Self::env) to add them. Defaults to 30 seconds for setup and 12 for shutdown.
    ///
    /// An empty command is reported as an error when you connect.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: vec![],
            env: vec![],
            setup_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(12),
        }
    }
    /// Add arguments to the command.
    ///
    /// Each string is one argument. Calling this more than once appends more arguments.
    ///
    /// ~~~rust
    /// use thinkwell::AgentConfig;
    ///
    /// let config = AgentConfig::new("my-acp-agent")
    ///     .args(["--transport", "stdio"])
    ///     .args(["--log-level", "warn"]);
    /// ~~~
    ///
    /// Arguments do not undergo shell quoting, variable expansion, or command substitution.
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }
    /// Set an environment variable for the agent process.
    ///
    /// For example, `config.env("LOG_LEVEL", "warn")` adds an override while leaving
    /// the rest of the inherited environment unchanged. Setting the same name again
    /// replaces its earlier value. Your application's own environment is not changed.
    ///
    /// Configurations can contain credentials; take care when logging them.
    pub fn env(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((name.into(), value.into()));
        self
    }
}
/// Adds `.connect().await` to an agent configuration or preset.
///
/// Import this trait when you want to connect an instance. The static
/// [Claude::connect](crate::agent::Claude::connect) convenience method does not need this import.
///
/// ~~~no_run
/// use thinkwell::{agent::Claude, Connect};
///
/// # async fn example() -> thinkwell::Result<()> {
/// let preset = Claude::default();
/// let agent = preset.connect().await?;
/// # Ok(())
/// # }
/// ~~~
///
/// # Adding your own preset
///
/// A preset is just a value that converts into [AgentConfig]. Implement that
/// conversion and the connection method becomes available automatically:
///
/// ~~~no_run
/// use thinkwell::{AgentConfig, Connect};
///
/// struct LocalAgent;
///
/// impl From<LocalAgent> for AgentConfig {
///     fn from(_: LocalAgent) -> Self {
///         AgentConfig::new("my-acp-agent").args(["--transport", "stdio"])
///     }
/// }
///
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = LocalAgent.connect().await?;
/// # Ok(())
/// # }
/// ~~~
pub trait Connect: Into<AgentConfig> + Sized {
    /// Start an agent using this value's configuration.
    ///
    /// This consumes the configuration and delegates to [Agent::connect]. Nothing
    /// starts until you poll the returned future, normally by awaiting it.
    fn connect(self) -> impl Future<Output = Result<Agent>> {
        Agent::connect(self)
    }
}
impl<T: Into<AgentConfig>> Connect for T {}

/// Limits on how much work a single request can do.
///
/// You can usually start with the defaults. Change these when, for example, an
/// interactive application needs a shorter wait or a workflow needs fewer tool
/// calls. Apply the settings with [Plan::options].
///
/// ~~~rust
/// use std::time::Duration;
/// use thinkwell::{Plan, RunOptions};
///
/// let options = RunOptions {
///     turn_timeout: Duration::from_secs(30),
///     max_tool_calls: 4,
///     ..RunOptions::default()
/// };
/// let plan = Plan::<String>::new()
///     .text("Suggest three names for a gardening club.")
///     .options(options);
/// ~~~
///
/// The defaults are 120 seconds for the whole request, 30 seconds per tool call,
/// 32 tool calls, and three invalid answers before giving up. These do not limit
/// model tokens or spending directly. All limits must be nonzero; durations must
/// fit the host clock. Invalid settings are rejected before the prompt is sent.
#[derive(Clone, Debug)]
pub struct RunOptions {
    /// How long to wait for the complete answer, including time spent running tools.
    ///
    /// When it expires, the request fails and the session and connection close.
    /// The default is 120 seconds.
    pub turn_timeout: Duration,
    /// How long one tool function may take, including checking its input and output.
    ///
    /// The default is 30 seconds. If it expires, Thinkwell stops waiting for the
    /// function and reports a tool error to the agent. The agent may continue.
    /// Changes the function already made are not undone.
    pub callback_timeout: Duration,
    /// How many application-tool calls to allow in one request; defaults to 32.
    ///
    /// Calls with invalid input and calls whose functions return errors still count.
    /// The next call after the limit ends the request with an error. Submitting the
    /// final answer uses a separate budget; calls rejected for an old turn identifier
    /// do not count.
    pub max_tool_calls: usize,
    /// How many invalid final answers to accept before ending the request with an error.
    ///
    /// The default of three gives the agent two opportunities to correct its first
    /// invalid answer. Missing values, type mismatches, and failed conversions count.
    /// Duplicates after an accepted answer and old turn identifiers do not count.
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
/// A description of Rust code that the agent can ask your application to run.
///
/// A tool has a name, a description, an input type `I`, and an output type `O`.
/// The description helps the agent decide when to use it. A Tool describes the
/// operation; [Plan::bind] supplies the function that does the work.
///
/// # Give an agent a word-counting tool
///
/// ~~~no_run
/// use thinkwell::{agent::Claude, Plan, Tool};
///
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// let count_words = Tool::<String, i64>::new(
///     "count_words",
///     "Count the words in a piece of text.",
/// );
/// let mut session = agent.session().tool(&count_words).open().await?;
///
/// let answer = Plan::<String>::new()
///     .text("Use count_words to count the words in 'the garden is growing'.")
///     .bind(&count_words, async |text: String| {
///         Ok(text.split_whitespace().count() as i64)
///     })
///     .run_in(&mut session).await?;
/// println!("{answer}");
/// # Ok(())
/// # }
/// ~~~
///
/// Notice the two steps: register the tool when opening the conversation, then
/// bind its function when making a request. Registering lets the agent know which
/// tools are available. Binding each time lets the function borrow the application
/// state it needs for that request. See [Plan::bind] for an example.
///
/// # Choosing input and output types
///
/// Inputs need `serde::Deserialize` and `schemars::JsonSchema`; outputs need
/// `serde::Serialize` and `schemars::JsonSchema`. String and i64 already implement
/// these. For your own structs, derive the corresponding traits.
///
/// Thinkwell uses these types to describe and check tool values. The currently
/// supported shapes are listed under [schema support](crate#schema-support).
/// Cloning a tool copies its description, not a function or any application state.
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
    /// Describe a tool the agent can call.
    ///
    /// Give it a short name and a description of when it is useful. See [Tool] for a
    /// complete example connecting the description to a Rust function.
    ///
    /// Names may contain ASCII letters, digits, underscores, and hyphens, and cannot
    /// be empty. The name `return_result` is reserved by Thinkwell. Invalid names or
    /// unsupported input/output types are reported later when opening a session or
    /// running a plan, before any prompt is sent.
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
                output: schema::output_contract::<O>()?,
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
/// Marks a [Plan] that has no tool functions yet.
///
/// Such a plan can be cloned and reused. You normally do not need to name this
/// type: [Plan::new] chooses it automatically, and [Plan::bind] changes it to [Bound].
pub struct Unbound;
/// Marks a [Plan] after you have attached a tool function.
///
/// These plans cannot be cloned because their functions may borrow application
/// state. You normally let Rust infer this type. Consuming or dropping the plan
/// releases its captures.
pub struct Bound;
pub(crate) struct Binding<'a> {
    pub descriptor: Descriptor,
    pub callback: Box<dyn Callback + 'a>,
}
/// A request you want to send to an agent, together with the type of answer you want.
///
/// Write the prompt with [text](Self::text), then use [run](Self::run) for one
/// fresh conversation or [run_in](Self::run_in) for a follow-up in an existing one. The type parameter `T` tells Thinkwell how to turn the
/// answer into a Rust value.
///
/// # Ask for a string
///
/// ~~~no_run
/// use thinkwell::{agent::Claude, Plan};
///
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// let title = Plan::<String>::new()
///     .text("Suggest a title for a beginner's gardening guide.")
///     .run(&agent).await?;
/// println!("{title}");
/// # Ok(())
/// # }
/// ~~~
///
/// # Ask for a structured answer
///
/// Describe the answer with a Rust type. `JsonSchema` lets Thinkwell tell the agent
/// what fields to produce; `Deserialize` lets it turn the answer into that type.
///
/// ~~~no_run
/// use schemars::JsonSchema;
/// use serde::Deserialize;
/// use thinkwell::{Plan, Session};
///
/// #[derive(Deserialize, JsonSchema)]
/// struct Suggestion {
///     title: String,
///     reason: String,
/// }
///
/// async fn suggest(session: &mut Session<'_>) -> thinkwell::Result<Suggestion> {
///     Plan::<Suggestion>::new()
///         .text("Suggest a title for a gardening guide and explain your choice.")
///         .run_in(session).await
/// }
/// ~~~
///
/// This helper expects a session without application tools. If your session has
/// tools, attach their functions with [bind](Self::bind) before each request.
///
/// A plan can be cloned until you bind tool functions to it. This is useful for
/// sharing instructions across requests; see [new](Self::new). Use [options](Self::options)
/// to change time and call limits. For a follow-up request, you may choose a different
/// answer type while keeping the same session and its conversation history.
pub struct Plan<'a, T, S = Unbound> {
    pub(crate) text: String,
    pub(crate) bindings: Vec<Binding<'a>>,
    pub(crate) error: Option<Error>,
    pub(crate) options: RunOptions,
    pub(crate) progress: Option<crate::progress::Sink>,
    types: PhantomData<fn() -> (T, S)>,
}
impl<T> Plan<'_, T> {
    /// Start writing a request with the default time and call limits.
    ///
    /// Creating a plan does not send anything. You can clone it to reuse instructions:
    ///
    /// ~~~rust
    /// use thinkwell::Plan;
    ///
    /// let base = Plan::<String>::new().text("Use a friendly tone.");
    /// let welcome = base.clone().text("Welcome a new gardening-club member.");
    /// let invitation = base.text("Invite members to a seed swap.");
    /// ~~~
    ///
    /// Thinkwell checks whether the answer type is supported when you run the plan.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            bindings: vec![],
            error: None,
            options: RunOptions::default(),
            progress: None,
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
            progress: None,
            types: PhantomData,
        }
    }
}
impl<'a, T, S> Plan<'a, T, S> {
    /// Show optional progress while retaining the normal typed result.
    ///
    /// Finish building the plan, attach a synchronous callback, then call
    /// [run](crate::ObservedPlan::run) or [run_in](crate::ObservedPlan::run_in).
    /// The callback can borrow local state; it need not be Send or static.
    ///
    /// ```no_run
    /// use thinkwell::{Plan, ProgressEvent, Agent};
    /// # async fn example(agent: &Agent) -> thinkwell::Result<()> {
    /// let answer = Plan::<String>::new().text("Suggest a recipe.")
    ///     .observe(|event| {
    ///         if let ProgressEvent::Message(message) = event {
    ///             eprint!("{}", message.text());
    ///         }
    ///     }).run(agent).await?;
    /// println!("{answer}");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Delivery is best-effort, with at most 128 queued events and 64 KiB of
    /// retained event data. Oversized or overflowing events are counted in
    /// [ProgressEvent::EventsDropped](crate::ProgressEvent::EventsDropped).
    /// Keep callbacks short; blocking one delays execution. See [ObservedPlan](crate::ObservedPlan)
    /// for panic, borrowing and cleanup behavior. No observer runs in the background.
    pub fn observe<F: FnMut(&crate::ProgressEvent)>(
        self,
        observer: F,
    ) -> crate::ObservedPlan<'a, T, S, F> {
        crate::ObservedPlan {
            plan: self,
            observer,
        }
    }

    /// Add instructions or context to the prompt.
    ///
    /// Call this more than once to build a prompt from separate pieces. Each call adds
    /// a newline after the text:
    ///
    /// ~~~rust
    /// use thinkwell::Plan;
    ///
    /// let topic = "growing tomatoes";
    /// let plan = Plan::<String>::new()
    ///     .text("Explain this topic to a beginner.")
    ///     .text(format!("Topic: {topic}"));
    /// ~~~
    ///
    /// Text is sent as written. This method does not escape it or distinguish trusted
    /// instructions from untrusted input.
    pub fn text(mut self, text: impl AsRef<str>) -> Self {
        self.text.push_str(text.as_ref());
        self.text.push('\n');
        self
    }
    /// Add a Rust value to the prompt as JSON.
    ///
    /// Use this to give the agent structured context, such as a list of attendees
    /// or a computation's result. Introduce the data with [`text`](Self::text):
    ///
    /// ~~~rust
    /// use serde::Serialize;
    /// use thinkwell::Plan;
    ///
    /// #[derive(Serialize)]
    /// struct Attendee<'a> {
    ///     name: &'a str,
    ///     availability: &'a str,
    /// }
    ///
    /// # fn example() -> thinkwell::Result<()> {
    /// let attendees = [Attendee {
    ///     name: "Sam",
    ///     availability: "Wednesday morning",
    /// }];
    /// let plan = Plan::<String>::new()
    ///     .text("Summarize the availability of these attendees:")
    ///     .json(&attendees)?;
    /// # Ok(())
    /// # }
    /// # example().unwrap();
    /// ~~~
    ///
    /// Serialization happens now; the plan stores the JSON text, not a borrow of
    /// the value. Only [`Serialize`] is required, not `JsonSchema`. The compact JSON
    /// is followed by a newline, with no labels or Markdown fences added.
    ///
    /// Returns [`ErrorKind::InvalidValue`] if JSON serialization fails. No request
    /// is sent by this method. JSON escapes strings but does not prevent an agent
    /// from treating their contents as instructions; it is not a trust boundary.
    pub fn json<V: Serialize + ?Sized>(self, value: &V) -> Result<Self> {
        let text = serde_json::to_string(value).map_err(|error| {
            Error::new(
                ErrorKind::InvalidValue,
                format!("Could not serialize prompt context: {error}"),
            )
        })?;
        Ok(self.text(text))
    }

    /// Choose time and call limits for this request.
    ///
    /// See [RunOptions] for an example. This replaces the plan's previous settings;
    /// cloning the plan copies them. Invalid settings are reported before prompting.
    pub fn options(mut self, options: RunOptions) -> Self {
        self.options = options;
        self
    }
    /// Provide the Rust function to run when the agent calls a tool.
    ///
    /// Use the same [Tool] you registered when opening the session. The function takes
    /// the tool's input type and returns `thinkwell::Result` of its output type.
    ///
    /// # Let a tool update your application state
    ///
    /// ~~~no_run
    /// use thinkwell::{agent::Claude, Plan, Tool};
    ///
    /// # async fn example() -> thinkwell::Result<()> {
    /// let agent = Claude::connect().await?;
    /// let add_item = Tool::<String, ()>::new("add_item", "Add an item to the shopping list.");
    /// let mut session = agent.session().tool(&add_item).open().await?;
    /// let mut shopping_list = Vec::new();
    ///
    /// let answer = Plan::<String>::new()
    ///     .text("Add tomatoes and basil to my shopping list, then summarize it.")
    ///     .bind(&add_item, async |item: String| {
    ///         shopping_list.push(item);
    ///         Ok(())
    ///     })
    ///     .run_in(&mut session).await?;
    ///
    /// println!("{answer}");
    /// println!("Saved items: {shopping_list:?}");
    /// # Ok(())
    /// # }
    /// ~~~
    ///
    /// The function borrows `shopping_list` while the request runs. After `.await`
    /// finishes and consumes the request, you can use the list normally again. Attach
    /// a function for every registered tool on every request, even if the agent may
    /// not call it.
    ///
    /// Return [Error::tool] if the operation cannot be performed. Thinkwell reports
    /// that error to the agent, which may try another approach. Any changes already
    /// made to your application state remain.
    ///
    /// # Reference details
    ///
    /// Functions may borrow local values and use non-Send values across await. Inputs
    /// are checked before calling the function, and outputs are checked after it returns.
    /// Missing, duplicate, or mismatched bindings are rejected before prompting.
    /// An error retained by Tool construction is also reported then.
    ///
    /// Binding makes the plan non-cloneable. Its captures are released when the plan
    /// or its execution future is consumed or dropped. If you retain a completed
    /// future in pinned storage, explicitly drop it before using its captures.
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
            progress: self.progress,
            types: PhantomData,
        }
    }
    /// Give this request a tool and the Rust function that implements it.
    ///
    /// Use this with [`run`](Self::run) when a tool is needed for a single turn.
    /// The callback can borrow local state, just like [`bind`](Self::bind):
    ///
    /// ~~~no_run
    /// use thinkwell::{agent::Claude, Plan};
    /// # async fn example() -> thinkwell::Result<()> {
    /// let agent = Claude::connect().await?;
    /// let mut items = Vec::new();
    /// let answer = Plan::<String>::new()
    ///     .text("Record tomatoes and basil, then summarize the shopping list.")
    ///     .tool("record", "Add an item to the shopping list", async |item: String| {
    ///         items.push(item);
    ///         Ok(())
    ///     })
    ///     .run(&agent).await?;
    /// println!("{answer}: {items:?}");
    /// # Ok(())
    /// # }
    /// ~~~
    ///
    /// Input and output types are inferred from the function. Inputs need
    /// `Deserialize` and `JsonSchema`; outputs need `Serialize` and `JsonSchema`.
    /// Invalid names or contracts are reported when executing the plan.
    /// Return [`Error::tool`] for failures the agent can respond to.
    ///
    /// Adding a callback makes the plan non-cloneable. Its captures stay borrowed
    /// until the plan or execution future is consumed or dropped. With
    /// [`run_in`](Self::run_in), this does not register a new tool: the description
    /// must match a tool already registered in the session, as with `bind`.
    pub fn tool<I, O, F>(
        self,
        name: impl Into<String>,
        description: impl Into<String>,
        f: F,
    ) -> Plan<'a, T, Bound>
    where
        I: DeserializeOwned + JsonSchema + 'static,
        O: Serialize + JsonSchema + 'static,
        F: AsyncFnMut(I) -> Result<O> + 'a,
    {
        self.bind(&Tool::<I, O>::new(name, description), f)
    }

    /// Ask one question in a fresh conversation, then release that conversation.
    ///
    /// Use this when you do not need follow-up history or a model override:
    ///
    /// ~~~no_run
    /// use thinkwell::{agent::Claude, Plan};
    /// # async fn example() -> thinkwell::Result<()> {
    /// let agent = Claude::connect().await?;
    /// let title = Plan::<String>::new()
    ///     .text("Suggest a title for a gardening guide.")
    ///     .run(&agent).await?;
    /// println!("{title}");
    /// // A second run starts a separate conversation on the same connection.
    /// let greeting = Plan::<String>::new().text("Say hello.").run(&agent).await?;
    /// # Ok(())
    /// # }
    /// ~~~
    ///
    /// Tools attached with [`tool`](Self::tool) or [`bind`](Self::bind) form this
    /// conversation's tool catalog. The temporary session uses the agent's default
    /// model and is dropped before returning. The agent remains connected after
    /// success. To retain history or choose a model, open a session explicitly and
    /// use [`run_in`](Self::run_in).
    ///
    /// Invalid contracts, duplicate tools, and invalid options fail before session
    /// setup. An already open session produces [`ErrorKind::Busy`]. A future that
    /// is never polled sends nothing. Once remote setup or a turn has started,
    /// failure or dropping the running future closes the connection, following the
    /// same rules as explicit sessions. Earlier callback effects are not rolled back.
    /// Captured state becomes available when the execution future is consumed or
    /// dropped; drop a retained pinned future before reusing its captures.
    ///
    /// Rust prevents using mutable captures before the execution ends:
    ///
    /// ```compile_fail,E0503
    /// use thinkwell::{Agent, Plan};
    /// async fn example(agent: &Agent) {
    ///     let mut count = 0;
    ///     let execution = Plan::<String>::new()
    ///         .tool("record", "Record a value", async |_: String| {
    ///             count += 1;
    ///             Ok(())
    ///         }).run(agent);
    ///     count += 1;
    ///     let _ = execution.await;
    /// }
    /// ```
    pub async fn run(self, agent: &Agent) -> Result<T>
    where
        T: DeserializeOwned + JsonSchema,
    {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        self.options.validate()?;
        schema::contract::<T>()?;
        let mut builder = agent.session();
        builder.progress = self.progress.clone();
        for binding in &self.bindings {
            builder = builder.descriptor(Ok(binding.descriptor.clone()));
        }
        let mut session = builder.open().await?;
        self.run_in(&mut session).await
    }

    /// Send this request to a conversation and wait for a typed answer.
    ///
    /// A request and its answer make up one *turn*. The session remembers earlier
    /// turns, so you can ask follow-up questions without repeating the conversation.
    /// The answer type can change from one turn to the next:
    ///
    /// ~~~no_run
    /// use thinkwell::{Plan, Session};
    ///
    /// async fn discuss(session: &mut Session<'_>) -> thinkwell::Result<()> {
    ///     let count = Plan::<i64>::new()
    ///         .text("How many days are in a week?")
    ///         .run_in(session).await?;
    ///     let explanation = Plan::<String>::new()
    ///         .text(format!("Explain that answer of {count} to a child."))
    ///         .run_in(session).await?;
    ///     println!("{explanation}");
    ///     Ok(())
    /// }
    /// ~~~
    ///
    /// This example uses a session with no application tools. For sessions with tools,
    /// see [bind](Self::bind). You can make only one request at a time in a session.
    ///
    /// # If something goes wrong
    ///
    /// Thinkwell checks the plan before sending it. A problem such as a missing tool
    /// function or an unsupported answer type leaves the session available for a
    /// corrected plan.
    ///
    /// Once a request has started, a failure closes both the session and its agent
    /// connection. Create a new connection to continue; Thinkwell does not silently
    /// start a new conversation for you. See [ErrorKind] for the kinds of failure.
    ///
    /// Dropping a running request also cancels your local work and closes the connection.
    /// It does not undo changes a tool has already made. Dropping a future that was
    /// never polled sends nothing and leaves the session unchanged.
    ///
    /// # When is the answer ready?
    ///
    /// Thinkwell checks the answer's shape and converts it into `T`. It waits for the
    /// agent to finish and for local tool work to settle before returning success;
    /// the first valid answer submitted by the agent cannot be replaced by a later one.
    /// The future must be polled to drive local tool functions, normally by awaiting it.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_context_composes_text_and_owns_its_snapshot() {
        let mut values = vec!["Sam\n\"quoted\"".to_owned()];
        let plan = Plan::<String>::new()
            .text("Attendees:")
            .json(values.as_slice())
            .unwrap()
            .text("Summarize them.");
        values.clear();
        assert_eq!(
            plan.text,
            "Attendees:\n[\"Sam\\n\\\"quoted\\\"\"]\nSummarize them.\n"
        );
    }

    #[test]
    fn json_context_preserves_bound_callbacks_and_plan_settings() {
        let tool = Tool::<String, String>::new("echo", "Echo input");
        let mut calls = 0;
        let mut plan = Plan::<String>::new().bind(&tool, async |value: String| {
            calls += 1;
            Ok(value)
        });
        plan.options.max_tool_calls = 7;
        plan.error = Some(Error::new(ErrorKind::Binding, "existing error"));
        let plan = plan.json("context").unwrap();
        assert_eq!(plan.bindings.len(), 1);
        assert_eq!(plan.options.max_tool_calls, 7);
        assert_eq!(plan.error.unwrap().kind, ErrorKind::Binding);
    }

    #[test]
    fn json_context_reports_serialization_failure_immediately() {
        struct Unserializable;
        impl Serialize for Unserializable {
            fn serialize<S: serde::Serializer>(
                &self,
                _: S,
            ) -> std::result::Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("context unavailable"))
            }
        }
        let Err(error) = Plan::<String>::new().json(&Unserializable) else {
            panic!("serialization must fail before execution");
        };
        assert_eq!(error.kind, ErrorKind::InvalidValue);
        assert!(error.message.contains("context unavailable"));
    }
}
