//! Built-in agents you can connect to with Thinkwell.
//!
//! Start with [Claude::connect], then open a conversation on the returned
//! [Agent]. For a custom launch command, use [AgentConfig].

use crate::{Agent, AgentConfig, Result};

/// The quickest way to connect to Claude.
///
/// Call [Claude::connect] to start a connection, then open a [Session](crate::Session) to begin a
/// conversation. You can use the agent's default model or select one when
/// [opening the session](crate::SessionBuilder::model).
///
/// ~~~no_run
/// use thinkwell::{agent::Claude, Plan};
///
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// let mut session = agent.session().open().await?;
/// let greeting = Plan::<String>::new()
///     .text("Say hello in one sentence.")
///     .run_in(&mut session).await?;
/// println!("{greeting}");
/// # Ok(())
/// # }
/// ~~~
///
/// # Before connecting
///
/// This preset runs `npx --offline --yes @zed-industries/claude-agent-acp@0.23.1`.
/// You need Node/npx, that adapter version already in your npm cache, and working
/// Claude authentication. Thinkwell does not install or authenticate it for you.
///
/// For a different launch command, use [configured](Self::configured). Import
/// [Connect](crate::Connect) to call `.connect()` on that configured instance.
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
    /// Choose a different command for starting Claude.
    ///
    /// For example, use an adapter executable already on your PATH:
    ///
    /// ~~~no_run
    /// use thinkwell::{AgentConfig, agent::Claude, Connect};
    ///
    /// # async fn example() -> thinkwell::Result<()> {
    /// let preset = Claude::configured(AgentConfig::new("claude-agent-acp"));
    /// let agent = preset.connect().await?;
    /// # Ok(())
    /// # }
    /// ~~~
    ///
    /// The supplied configuration replaces the default command and all its arguments.
    /// It is not merged with the npx defaults.
    pub fn configured(config: AgentConfig) -> Self {
        Self { config }
    }
    /// Start Claude using the default configuration.
    ///
    /// See [Claude] for the prerequisites and a first conversation. This is shorthand
    /// for `Agent::connect(Claude::default()).await`. Connection errors and runtime
    /// requirements are described on [Agent::connect].
    pub async fn connect() -> Result<Agent> {
        Agent::connect(Self::default()).await
    }
}
impl From<Claude> for AgentConfig {
    fn from(value: Claude) -> Self {
        value.config
    }
}
