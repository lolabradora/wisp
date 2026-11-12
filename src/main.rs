use std::io::{IsTerminal as _, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Context as _;
use clap::{Args, Parser, Subcommand, ValueEnum};
use tracing_subscriber::EnvFilter;
use wisp::context::{Context, Sources};
use wisp::daemon::{self, Config};
use wisp::model::{HealedPrompt, LlamaClient};
use wisp::screen::Terminal;
use wisp::service::ModelSize;
use wisp::{doctor, paths, setup, status};

const ZSH_PLUGIN: &str = include_str!("../shell/wisp.zsh");

/// Ghost-text command suggestions for zsh from a local model that reads your terminal.
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the shell plugin. Add `eval "$(wisp init zsh)"` to the end of ~/.zshrc.
    Init {
        shell: Shell,
        #[command(flatten)]
        socket: SocketArg,
    },
    /// Download the model and keep llama-server running in the background. Run it again to
    /// switch models.
    Setup {
        /// Qwen2.5-Coder size: 1.5b is faster, 3b suggests better.
        #[arg(long, value_enum, default_value_t)]
        model: ModelSize,
        /// Port llama-server listens on, on 127.0.0.1.
        #[arg(long, default_value_t = setup::DEFAULT_PORT)]
        port: u16,
        /// Also show the menu bar item at login (macOS).
        #[arg(long)]
        menubar: bool,
    },
    /// Remove the background services that `wisp setup` installed. Models and the binary stay.
    Uninstall,
    /// Check everything a suggestion depends on, with a fix for each problem.
    Doctor(Endpoints),
    /// Run the daemon the shell plugin talks to. The plugin starts it on demand.
    Daemon {
        #[command(flatten)]
        endpoints: Endpoints,
        /// How long typing must pause before the model is asked, in milliseconds.
        #[arg(long, env = "WISP_DEBOUNCE_MS", default_value_t = 60)]
        debounce_ms: u64,
    },
    /// Show whether llama-server and the daemon run, and the latest suggestions.
    Status {
        #[command(flatten)]
        endpoints: Endpoints,
        /// Print the report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Stop suggesting until `wisp resume`. Survives daemon restarts.
    Pause(SocketArg),
    /// Undo `wisp pause`.
    Resume(SocketArg),
    /// Show wisp's status in the macOS menu bar.
    Menubar(Endpoints),
    /// Suggest a completion for BUFFER from this directory and terminal, for debugging.
    Complete {
        buffer: String,
        /// Print the prompt sent to the model.
        #[arg(long)]
        show_prompt: bool,
        #[command(flatten)]
        endpoints: Endpoints,
    },
}

#[derive(Args)]
struct SocketArg {
    /// Defaults to ~/.cache/wisp/wisp.sock.
    #[arg(long)]
    socket: Option<PathBuf>,
}

impl SocketArg {
    fn resolve(self) -> anyhow::Result<PathBuf> {
        self.socket.map_or_else(paths::default_socket, Ok)
    }
}

#[derive(Args)]
struct Endpoints {
    #[command(flatten)]
    socket: SocketArg,
    /// Base URL of the llama-server that generates suggestions.
    #[arg(long, env = "WISP_LLM_URL", default_value = "http://127.0.0.1:8012")]
    llm_url: String,
}

impl Endpoints {
    fn resolve(self) -> anyhow::Result<(PathBuf, LlamaClient)> {
        Ok((self.socket.resolve()?, LlamaClient::new(&self.llm_url)?))
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum Shell {
    Zsh,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("WISP_LOG").unwrap_or_else(|_| EnvFilter::new("wisp=info")),
        )
        .with_writer(std::io::stderr)
        // The daemon and menu bar log to files, where color codes are noise.
        .with_ansi(std::io::stderr().is_terminal())
        .init();

    match Cli::parse().command {
        // AppKit needs the main thread, so the menu bar runs outside the tokio runtime.
        Command::Menubar(endpoints) => run_menubar(endpoints),
        command @ (Command::Init { .. }
        | Command::Setup { .. }
        | Command::Uninstall
        | Command::Doctor(_)
        | Command::Daemon { .. }
        | Command::Status { .. }
        | Command::Pause(_)
        | Command::Resume(_)
        | Command::Complete { .. }) => tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .context("starting the tokio runtime")?
            .block_on(run(command)),
    }
}

#[cfg(target_os = "macos")]
fn run_menubar(endpoints: Endpoints) -> anyhow::Result<()> {
    let (socket, model) = endpoints.resolve()?;
    wisp::menubar::run(&socket, model)
}

#[cfg(not(target_os = "macos"))]
fn run_menubar(_: Endpoints) -> anyhow::Result<()> {
    anyhow::bail!("`wisp menubar` is only available on macOS; `wisp status` shows the same report")
}

async fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Init { shell: Shell::Zsh, socket } => print_zsh_plugin(&socket.resolve()?),
        Command::Setup { model, port, menubar } => {
            setup::setup(setup::Options { model, port, menubar }, &mut std::io::stdout()).await
        }
        Command::Uninstall => setup::uninstall(&mut std::io::stdout()).await,
        Command::Doctor(endpoints) => run_doctor(endpoints).await,
        Command::Daemon { endpoints, debounce_ms } => {
            let (socket, model) = endpoints.resolve()?;
            let paused_flag = paths::paused_flag(&socket);
            let debounce = Duration::from_millis(debounce_ms);
            daemon::run(Config { socket, model, debounce, paused_flag }).await
        }
        Command::Status { endpoints, json } => {
            let (socket, model) = endpoints.resolve()?;
            let report = status::collect(&socket, &model).await;
            let text = if json {
                serde_json::to_string_pretty(&report)? + "\n"
            } else {
                report.summary().to_text()
            };
            std::io::stdout().write_all(text.as_bytes()).context("writing status")
        }
        Command::Pause(socket) => set_paused(&socket.resolve()?, true),
        Command::Resume(socket) => set_paused(&socket.resolve()?, false),
        Command::Complete { buffer, show_prompt, endpoints } => {
            complete_once(&buffer, show_prompt, &endpoints.llm_url).await
        }
        Command::Menubar(_) => anyhow::bail!("the menu bar must run on the main thread"),
    }
}

/// Prints the checklist, then fails if a check did, so scripts can rely on the exit status.
async fn run_doctor(endpoints: Endpoints) -> anyhow::Result<()> {
    let llm_url = endpoints.llm_url.clone();
    let (socket, model) = endpoints.resolve()?;
    let checks = doctor::Findings::gather(&socket, &model, &llm_url).await?.checks();
    std::io::stdout().write_all(doctor::render(&checks).as_bytes()).context("writing checklist")?;
    match doctor::failures(&checks) {
        0 => Ok(()),
        1 => anyhow::bail!("1 check failed"),
        n => anyhow::bail!("{n} checks failed"),
    }
}

fn set_paused(socket: &Path, paused: bool) -> anyhow::Result<()> {
    paths::set_paused(socket, paused)?;
    let state = if paused { "paused; `wisp resume` turns them back on" } else { "on" };
    writeln!(std::io::stdout(), "suggestions {state}").context("writing to stdout")
}

fn print_zsh_plugin(socket: &Path) -> anyhow::Result<()> {
    let exe = std::env::current_exe().context("locating the wisp binary")?;
    let plugin = ZSH_PLUGIN
        .replace("__WISP_BIN__", &shell_quote(&exe.display().to_string()))
        .replace("__WISP_SOCKET__", &shell_quote(&socket.display().to_string()));
    std::io::stdout().write_all(plugin.as_bytes()).context("writing plugin to stdout")
}

async fn complete_once(buffer: &str, show_prompt: bool, llm_url: &str) -> anyhow::Result<()> {
    let sources = Sources {
        cwd: std::env::current_dir().context("reading current directory")?,
        terminal: Terminal::from_env(),
        histfile: std::env::var_os("HISTFILE")
            .map(PathBuf::from)
            .or_else(|| std::env::home_dir().map(|home| home.join(".zsh_history"))),
    };
    let gather_started = Instant::now();
    let context = Context::gather(&sources, &[]).await;
    let gather_time = gather_started.elapsed();
    let transcript = context.transcript();

    let mut out = std::io::stdout().lock();
    if show_prompt {
        let healed = HealedPrompt::new(&transcript, buffer);
        writeln!(out, "{}\n--- grammar: {}", healed.prompt, healed.grammar)?;
        let source = sources.terminal.source().map_or("none", |source| source.name());
        writeln!(out, "--- screen source: {source}")?;
    }
    let model = LlamaClient::new(llm_url)?;
    for attempt in ["cold", "cached"] {
        let started = Instant::now();
        let suggestion = model.complete_command(&transcript, buffer).await?;
        writeln!(
            out,
            "{attempt}: {suggestion:?} in {:?} (context gathered in {gather_time:?})",
            started.elapsed()
        )?;
    }
    Ok(())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}
