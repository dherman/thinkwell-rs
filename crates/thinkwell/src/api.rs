use crate::{Agent, Error, ErrorKind, Result, Session, schema};
use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{future::Future, marker::PhantomData, pin::Pin, time::Duration};

/// Owned command configuration; custom presets can implement `Into<AgentConfig>`.
#[derive(Clone, Debug)]
pub struct AgentConfig {
    pub(crate) command: String,
    pub(crate) args: Vec<String>,
    pub(crate) env: Vec<(String, String)>,
    pub setup_timeout: Duration,
    pub shutdown_timeout: Duration,
}
impl AgentConfig {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: vec![],
            env: vec![],
            setup_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(12),
        }
    }
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }
    pub fn env(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((name.into(), value.into()));
        self
    }
}
/// Pinned ACP adapter; installation/cache and authentication are caller prerequisites.
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
    pub fn configured(config: AgentConfig) -> Self {
        Self { config }
    }
    pub async fn connect() -> Result<Agent> {
        Agent::connect(Self::default()).await
    }
}
impl From<Claude> for AgentConfig {
    fn from(value: Claude) -> Self {
        value.config
    }
}
/// Import to use configured_preset.connect().await.
pub trait Connect: Into<AgentConfig> + Sized {
    fn connect(self) -> impl Future<Output = Result<Agent>> {
        Agent::connect(self)
    }
}
impl<T: Into<AgentConfig>> Connect for T {}

#[derive(Clone, Debug)]
pub struct RunOptions {
    pub turn_timeout: Duration,
    pub callback_timeout: Duration,
    pub max_tool_calls: usize,
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
/// A stable contract; implementations are bound separately for each turn.
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
pub struct Unbound;
pub struct Bound;
pub(crate) struct Binding<'a> {
    pub descriptor: Descriptor,
    pub callback: Box<dyn Callback + 'a>,
}
/// Unbound plans are cloneable; bound plans own potentially borrowed callbacks.
pub struct Plan<'a, T, S = Unbound> {
    pub(crate) text: String,
    pub(crate) bindings: Vec<Binding<'a>>,
    pub(crate) error: Option<Error>,
    pub(crate) options: RunOptions,
    types: PhantomData<fn() -> (T, S)>,
}
impl<T> Plan<'_, T> {
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
    pub fn text(mut self, text: impl AsRef<str>) -> Self {
        self.text.push_str(text.as_ref());
        self.text.push('\n');
        self
    }
    pub fn options(mut self, options: RunOptions) -> Self {
        self.options = options;
        self
    }
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
    /// Execution owns the exclusive session borrow and its bound callbacks.
    ///
    /// ```compile_fail,E0499
    /// use thinkwell::{Plan,Session};
    /// async fn overlap(s:&mut Session<'_>) {
    ///     let a=Plan::<String>::new().run_in(s);
    ///     let b=Plan::<String>::new().run_in(s);
    ///     let _=a.await; let _=b.await;
    /// }
    /// ```
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
