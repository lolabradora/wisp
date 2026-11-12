//! `wisp setup` and `wisp uninstall`: download the model and keep llama-server running as a
//! background service (a launchd agent on macOS, a systemd user unit elsewhere).
//!
//! The files that get installed are built in [`crate::service`]; this module only runs the
//! steps around them.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context as _, bail};
use tokio::process::Command;

use crate::model::LlamaClient;
use crate::service::{self, LlamaService, ModelSize};
use crate::{doctor, paths};

const HEALTH_POLL: Duration = Duration::from_millis(500);
/// Loading the 3B weights from a cold disk takes a few seconds; a minute means it is broken.
const SERVICE_START_TIMEOUT: Duration = Duration::from_secs(60);
/// The port the daemon expects llama-server on unless `WISP_LLM_URL` says otherwise.
pub const DEFAULT_PORT: u16 = 8012;

/// What `wisp setup` was asked to do.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub model: ModelSize,
    pub port: u16,
    /// Also show `wisp menubar` at login (macOS only).
    pub menubar: bool,
}

/// Downloads the model if needed, installs the llama-server service and waits until it answers.
/// Running it again replaces the service, which is how the model or port is changed.
///
/// # Errors
///
/// Returns an error if llama-server is not installed, the port is taken by something else, the
/// download fails, or the service cannot be installed or does not become healthy.
pub async fn setup(options: Options, out: &mut impl Write) -> anyhow::Result<()> {
    if options.menubar && !cfg!(target_os = "macos") {
        bail!("--menubar needs macOS; the menu bar item is not available on this system");
    }
    let server = find_llama_server()?;
    writeln!(out, "llama-server: {}", server.display())?;

    let url = format!("http://127.0.0.1:{}", options.port);
    let client = LlamaClient::new(&url)?;
    // The old service holds the port the download run and the new service need.
    platform::stop_llama().await?;
    if client.status().await.healthy {
        bail!(
            "a llama-server that wisp did not install already answers on {url}; \
             stop it or pick another port with --port"
        );
    }

    let home = paths::home()?;
    let model = options.model;
    if service::model_is_cached(&service::hub_dirs(|name| std::env::var_os(name), &home), model) {
        writeln!(out, "model: {} is already downloaded", model.repo())?;
    } else {
        download_model(&server, options, &client, out).await?;
    }

    let service = LlamaService { server, model, port: options.port, env: cache_env() };
    let installed = platform::install_llama(&service).await?;
    writeln!(out, "service: installed {installed}, waiting for llama-server to load the model")?;
    wait_until_healthy(&client).await?;
    writeln!(
        out,
        "llama-server ({}) is running on 127.0.0.1:{} (log: {})",
        model.label(),
        options.port,
        paths::llama_log_hint()?
    )?;

    #[cfg(target_os = "macos")]
    if options.menubar {
        let installed = platform::install_menubar().await?;
        writeln!(out, "menu bar: installed {installed}")?;
    }
    write_next_steps(options.port, out)
}

/// Removes the services `wisp setup` installed. Models and the binary stay.
///
/// # Errors
///
/// Returns an error if a service file cannot be removed.
pub async fn uninstall(out: &mut impl Write) -> anyhow::Result<()> {
    let removed = platform::uninstall().await?;
    if removed.is_empty() {
        writeln!(out, "no wisp services were installed")?;
    }
    for service in removed {
        writeln!(out, "removed {service}")?;
    }
    writeln!(
        out,
        "Downloaded models and the wisp binary were left in place.\n\
         To finish, remove the `wisp init zsh` line from {}.",
        paths::zshrc()?.display()
    )?;
    Ok(())
}

fn find_llama_server() -> anyhow::Result<PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    service::find_on_path("llama-server", &path)
        .with_context(|| format!("llama-server not found on PATH; {}", install_hint()))
}

/// How to get llama-server on this system.
#[must_use]
pub fn install_hint() -> &'static str {
    if cfg!(target_os = "macos") {
        "install it with: brew install llama.cpp"
    } else {
        "install it with `brew install llama.cpp`, or unpack a release from \
         https://github.com/ggml-org/llama.cpp/releases and put llama-server on PATH"
    }
}

/// The cache-location variables set in this shell, for the service to see too.
fn cache_env() -> Vec<(String, String)> {
    service::CACHE_VARS
        .into_iter()
        .filter_map(|name| Some((name.to_owned(), std::env::var(name).ok()?)))
        .filter(|(_, value)| !value.is_empty())
        .collect()
}

/// Runs llama-server in the foreground with the model's preset, which downloads the weights,
/// and stops it once it serves. The service itself runs offline and could not download them.
async fn download_model(
    server: &Path,
    options: Options,
    client: &LlamaClient,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    let preset = options.model.preset();
    writeln!(
        out,
        "model: downloading {} (llama-server reports progress below; this happens once)",
        options.model.repo()
    )?;
    out.flush()?;
    let mut child = Command::new(server)
        .args([&preset, "--host", "127.0.0.1", "--port", &options.port.to_string()])
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("starting {}", server.display()))?;
    // No deadline: the download takes as long as the connection needs. A failure ends the
    // process, which ends the wait.
    loop {
        tokio::time::sleep(HEALTH_POLL).await;
        if let Some(status) = child.try_wait().context("waiting for llama-server")? {
            bail!("`llama-server {preset}` stopped before the model was ready ({status})");
        }
        if client.status().await.healthy {
            break;
        }
    }
    child.kill().await.context("stopping the download run of llama-server")?;
    writeln!(out, "model: downloaded")?;
    Ok(())
}

async fn wait_until_healthy(client: &LlamaClient) -> anyhow::Result<()> {
    let started = tokio::time::Instant::now();
    while started.elapsed() < SERVICE_START_TIMEOUT {
        if client.status().await.healthy {
            return Ok(());
        }
        tokio::time::sleep(HEALTH_POLL).await;
    }
    bail!(
        "llama-server did not become healthy within {}s; see {}",
        SERVICE_START_TIMEOUT.as_secs(),
        paths::llama_log_hint()?
    )
}

fn write_next_steps(port: u16, out: &mut impl Write) -> anyhow::Result<()> {
    let zshrc = paths::zshrc()?;
    let loaded = std::fs::read_to_string(&zshrc).is_ok_and(|text| doctor::loads_wisp(&text));
    writeln!(out)?;
    if loaded {
        writeln!(out, "{} already loads wisp; open a new shell to use it.", zshrc.display())?;
    } else {
        writeln!(out, "Add this line to the end of {}, then open a new shell:", zshrc.display())?;
        writeln!(out, "  eval \"$(wisp init zsh)\"")?;
    }
    if port != DEFAULT_PORT {
        writeln!(
            out,
            "wisp looks for llama-server on port {DEFAULT_PORT}; put this before that line:\n  \
             export WISP_LLM_URL=http://127.0.0.1:{port}"
        )?;
    }
    Ok(())
}

/// Runs a command that must succeed, keeping its stderr for the error message.
async fn run(program: &str, args: &[&str]) -> anyhow::Result<()> {
    let description = format!("{program} {}", args.join(" "));
    let output = Command::new(program)
        .args(args)
        .output()
        .await
        .with_context(|| format!("running {description}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{description} failed ({}): {}", output.status, stderr.trim());
    }
    Ok(())
}

/// Runs a command whose failure is an answer rather than an error.
async fn succeeds(program: &str, args: &[&str]) -> bool {
    Command::new(program).args(args).output().await.is_ok_and(|output| output.status.success())
}

async fn write_service_file(path: &Path, contents: &str) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir)
            .await
            .with_context(|| format!("creating {}", dir.display()))?;
    }
    tokio::fs::write(path, contents).await.with_context(|| format!("writing {}", path.display()))
}

/// Removes a file; `false` when it was not there.
async fn remove_service_file(path: &Path) -> anyhow::Result<bool> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err).with_context(|| format!("removing {}", path.display())),
    }
}

/// launchd agents in the user's GUI session.
#[cfg(target_os = "macos")]
mod platform {
    use std::path::Path;
    use std::time::Duration;

    use anyhow::Context as _;

    use super::{remove_service_file, run, succeeds, write_service_file};
    use crate::paths::{self, LLAMA_AGENT, MENUBAR_AGENT};
    use crate::service::{self, LlamaService};
    use crate::status::{launchd_domain, launchd_target};

    pub async fn stop_llama() -> anyhow::Result<()> {
        unload_agent(LLAMA_AGENT).await.map(drop)
    }

    pub async fn install_llama(service: &LlamaService) -> anyhow::Result<String> {
        let log = paths::llama_log()?;
        install_agent(LLAMA_AGENT, &service.launchd_plist(&log), &log).await
    }

    pub async fn install_menubar() -> anyhow::Result<String> {
        let wisp = std::env::current_exe().context("locating the wisp binary")?;
        let log = paths::menubar_log()?;
        install_agent(MENUBAR_AGENT, &service::menubar_plist(&wisp, &log), &log).await
    }

    pub async fn uninstall() -> anyhow::Result<Vec<String>> {
        let mut removed = Vec::new();
        for label in [LLAMA_AGENT, MENUBAR_AGENT] {
            let was_loaded = unload_agent(label).await?;
            let had_plist = remove_service_file(&paths::launch_agent(label)?).await?;
            if was_loaded || had_plist {
                removed.push(format!("launchd agent {label}"));
            }
        }
        Ok(removed)
    }

    /// Writes the plist and (re)loads the agent, replacing any loaded copy.
    async fn install_agent(label: &str, plist: &str, log: &Path) -> anyhow::Result<String> {
        let path = paths::launch_agent(label)?;
        write_service_file(&path, plist).await?;
        if let Some(dir) = log.parent() {
            tokio::fs::create_dir_all(dir)
                .await
                .with_context(|| format!("creating {}", dir.display()))?;
        }
        let path = path.display().to_string();
        // launchd reports a malformed plist only as "Input/output error"; plutil says why.
        run("plutil", &["-lint", &path]).await?;
        unload_agent(label).await?;
        run("launchctl", &["bootstrap", &launchd_domain()?, &path]).await?;
        Ok(format!("launchd agent {label}"))
    }

    /// Stops and unloads the agent; `false` when it was not loaded.
    async fn unload_agent(label: &str) -> anyhow::Result<bool> {
        let target = launchd_target(label)?;
        if !succeeds("launchctl", &["print", &target]).await {
            return Ok(false);
        }
        succeeds("launchctl", &["bootout", &target]).await;
        // bootout returns before the old agent is gone, and bootstrap fails while it is still
        // loaded.
        for _ in 0..50 {
            if !succeeds("launchctl", &["print", &target]).await {
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        Ok(true)
    }
}

/// A systemd user unit.
#[cfg(not(target_os = "macos"))]
mod platform {
    use anyhow::Context as _;

    use super::{remove_service_file, run, succeeds, write_service_file};
    use crate::paths::{self, LLAMA_UNIT};
    use crate::service::LlamaService;

    const NEEDS_SYSTEMD: &str = "wisp setup installs a systemd user service, which needs a working `systemctl --user`; \
         without systemd, start llama-server yourself (see the unit file for the command)";

    pub async fn stop_llama() -> anyhow::Result<()> {
        // Fails when the unit does not exist yet, which is the first run.
        succeeds("systemctl", &["--user", "stop", LLAMA_UNIT]).await;
        Ok(())
    }

    pub async fn install_llama(service: &LlamaService) -> anyhow::Result<String> {
        write_service_file(&paths::systemd_unit()?, &service.systemd_unit()).await?;
        run("systemctl", &["--user", "daemon-reload"]).await.context(NEEDS_SYSTEMD)?;
        run("systemctl", &["--user", "enable", "--now", LLAMA_UNIT]).await?;
        Ok(format!("systemd user unit {LLAMA_UNIT}"))
    }

    pub async fn uninstall() -> anyhow::Result<Vec<String>> {
        let unit = paths::systemd_unit()?;
        if !tokio::fs::try_exists(&unit).await.unwrap_or(false) {
            return Ok(Vec::new());
        }
        // Stops the service and removes the symlink that starts it at login.
        succeeds("systemctl", &["--user", "disable", "--now", LLAMA_UNIT]).await;
        remove_service_file(&unit).await?;
        succeeds("systemctl", &["--user", "daemon-reload"]).await;
        Ok(vec![format!("systemd user unit {LLAMA_UNIT}")])
    }
}
