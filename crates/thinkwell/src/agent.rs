//! Built-in agents you can connect to with Thinkwell.
//!
//! Start with [Claude::connect] or [Codex::connect], then use the returned
//! [Agent] to run a plan or open a conversation. For a custom launch command,
//! use [AgentConfig].

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

/// Connect to Codex and give it typed Rust work.
///
/// A plan can ask a question and return an ordinary Rust value:
///
/// ~~~no_run
/// use thinkwell::{agent::Codex, Plan};
///
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Codex::connect().await?;
/// let suggestions = Plan::<Vec<String>>::new()
///     .text("Suggest three names for a community garden.")
///     .run(&agent).await?;
/// for name in suggestions {
///     println!("{name}");
/// }
/// # Ok(())
/// # }
/// ~~~
///
/// To keep conversation history, open a [Session](crate::Session) and use
/// [Plan::run_in](crate::Plan::run_in). You can choose a model with
/// [SessionBuilder::model](crate::SessionBuilder::model); otherwise Codex chooses
/// its default. Model names depend on your Codex account and configuration.
///
/// # Before connecting
///
/// Install Node/npx and cache `@agentclientprotocol/codex-acp@2.8.0`, for example
/// with `npm exec --yes --package=@agentclientprotocol/codex-acp@2.8.0 -- node --version`.
/// This installs the adapter package but runs only Node's version command. Authenticate
/// Codex separately. This preset launches the adapter with `npx --offline --yes`, so
/// connecting does not install it. The adapter starts Codex App Server; its own
/// dependency determines the Codex version.
///
/// The preset selects the adapter's read-only mode. Thinkwell approves only its
/// declared tools when the agent asks permission. Codex may perform native reads
/// without asking; this is not a tools-only sandbox. Your Codex configuration
/// still applies, including other MCP servers. Do not configure a server named
/// `thinkwell`: that name is reserved for the tools in your conversation.
#[derive(Clone, Debug)]
pub struct Codex {
    config: AgentConfig,
}
impl Default for Codex {
    fn default() -> Self {
        Self {
            config: AgentConfig::new("npx")
                .args(["--offline", "--yes", "@agentclientprotocol/codex-acp@2.8.0"])
                .env("INITIAL_AGENT_MODE", "read-only"),
        }
    }
}
impl Codex {
    /// Use your own Codex adapter command and settings.
    ///
    /// For example, launch an adapter already installed on your PATH:
    ///
    /// ~~~no_run
    /// use thinkwell::{agent::Codex, AgentConfig, Connect};
    ///
    /// # async fn example() -> thinkwell::Result<()> {
    /// let preset = Codex::configured(
    ///     AgentConfig::new("codex-acp").env("INITIAL_AGENT_MODE", "read-only")
    /// );
    /// let agent = preset.connect().await?;
    /// # Ok(())
    /// # }
    /// ~~~
    ///
    /// This replaces the entire default configuration, including its read-only
    /// mode setting. Import [Connect](crate::Connect) for instance `.connect()`.
    pub fn configured(config: AgentConfig) -> Self {
        Self { config }
    }
    /// Start a connection using the default Codex adapter configuration.
    ///
    /// See [Codex] for setup and a first example. This is shorthand for
    /// `Agent::connect(Codex::default()).await`; it does not start a conversation
    /// until you run a plan or open a session. See [Agent::connect] for errors
    /// and runtime requirements.
    pub async fn connect() -> Result<Agent> {
        Agent::connect(Self::default()).await
    }
}
impl From<Codex> for AgentConfig {
    fn from(value: Codex) -> Self {
        value.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_launch_and_configuration_replacement() {
        let config: AgentConfig = Codex::default().into();
        assert_eq!(config.command, "npx");
        assert_eq!(
            config.args,
            ["--offline", "--yes", "@agentclientprotocol/codex-acp@2.8.0"]
        );
        assert_eq!(
            config.env,
            [("INITIAL_AGENT_MODE".into(), "read-only".into())]
        );
        let custom: AgentConfig =
            Codex::configured(AgentConfig::new("custom").args(["arg"])).into();
        assert_eq!(custom.command, "custom");
        assert_eq!(custom.args, ["arg"]);
        assert!(custom.env.is_empty());
    }
}
