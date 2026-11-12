//! Where wisp keeps its files. Everything the daemon owns sits next to its socket, so a custom
//! `--socket` moves the pause flag and the log along with it.

use std::path::{Path, PathBuf};

use anyhow::Context as _;

/// launchd label of the llama-server agent that `wisp setup` installs on macOS.
pub const LLAMA_AGENT: &str = "dev.wisp.llama-server";

/// launchd label of the menu bar agent installed by `wisp setup --menubar`.
pub const MENUBAR_AGENT: &str = "dev.wisp.menubar";

/// systemd user unit that `wisp setup` installs on Linux.
pub const LLAMA_UNIT: &str = "wisp-llama-server.service";

/// A fixed per-user path, so every shell finds the same daemon even when tools such as
/// nix-shell or direnv change `TMPDIR`.
///
/// # Errors
///
/// Returns an error if `HOME` is not set.
pub fn default_socket() -> anyhow::Result<PathBuf> {
    Ok(home().context("pass --socket explicitly")?.join(".cache/wisp/wisp.sock"))
}

/// While this file exists the daemon makes no suggestions.
#[must_use]
pub fn paused_flag(socket: &Path) -> PathBuf {
    socket.with_file_name("paused")
}

/// Creates or removes the pause flag next to `socket`.
///
/// # Errors
///
/// Returns an error if the flag cannot be written or removed.
pub fn set_paused(socket: &Path, paused: bool) -> anyhow::Result<()> {
    let flag = paused_flag(socket);
    if paused {
        if let Some(dir) = flag.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        std::fs::write(&flag, "").with_context(|| format!("writing {}", flag.display()))
    } else {
        match std::fs::remove_file(&flag) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err).with_context(|| format!("removing {}", flag.display())),
        }
    }
}

/// The zsh plugin sends the daemon's output here (see `shell/wisp.zsh`).
#[must_use]
pub fn daemon_log(socket: &Path) -> PathBuf {
    socket.with_extension("log")
}

/// The file launchd sends llama-server's output to. Only macOS has one; systemd keeps the
/// output in the journal (see [`llama_log_hint`]).
///
/// # Errors
///
/// Returns an error if `HOME` is not set.
#[cfg(target_os = "macos")]
pub fn llama_log() -> anyhow::Result<PathBuf> {
    Ok(home()?.join("Library/Logs/wisp-llama-server.log"))
}

/// Where to read llama-server's output, worded for messages: the log file on macOS, the
/// `journalctl` command elsewhere.
///
/// # Errors
///
/// Returns an error if `HOME` is not set.
pub fn llama_log_hint() -> anyhow::Result<String> {
    #[cfg(target_os = "macos")]
    let hint = llama_log()?.display().to_string();
    #[cfg(not(target_os = "macos"))]
    let hint = format!("journalctl --user -u {}", LLAMA_UNIT.trim_end_matches(".service"));
    Ok(hint)
}

/// # Errors
///
/// Returns an error if `HOME` is not set.
pub fn menubar_log() -> anyhow::Result<PathBuf> {
    Ok(home()?.join("Library/Logs/wisp-menubar.log"))
}

/// The plist launchd reads for the agent `label`.
///
/// # Errors
///
/// Returns an error if `HOME` is not set.
pub fn launch_agent(label: &str) -> anyhow::Result<PathBuf> {
    Ok(home()?.join(format!("Library/LaunchAgents/{label}.plist")))
}

/// The unit file systemd reads for the llama-server user service.
///
/// # Errors
///
/// Returns an error if `HOME` is not set.
pub fn systemd_unit() -> anyhow::Result<PathBuf> {
    Ok(home()?.join(".config/systemd/user").join(LLAMA_UNIT))
}

/// The file zsh reads for interactive shells, where the `wisp init zsh` line belongs.
///
/// # Errors
///
/// Returns an error if neither `ZDOTDIR` nor `HOME` is set.
pub fn zshrc() -> anyhow::Result<PathBuf> {
    let dir = match std::env::var_os("ZDOTDIR").filter(|dir| !dir.is_empty()) {
        Some(dir) => PathBuf::from(dir),
        None => home()?,
    };
    Ok(dir.join(".zshrc"))
}

/// # Errors
///
/// Returns an error if `HOME` is not set.
pub fn home() -> anyhow::Result<PathBuf> {
    std::env::home_dir().context("HOME is not set")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pausing_and_resuming_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("new/wisp.sock");
        set_paused(&socket, false).unwrap();
        set_paused(&socket, true).unwrap();
        set_paused(&socket, true).unwrap();
        assert!(paused_flag(&socket).exists());
        set_paused(&socket, false).unwrap();
        set_paused(&socket, false).unwrap();
        assert!(!paused_flag(&socket).exists());
    }

    #[test]
    fn daemon_files_live_next_to_the_socket() {
        let socket = Path::new("/x/.cache/wisp/wisp.sock");
        assert_eq!(paused_flag(socket), Path::new("/x/.cache/wisp/paused"));
        assert_eq!(daemon_log(socket), Path::new("/x/.cache/wisp/wisp.log"));
    }
}
