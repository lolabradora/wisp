//! `wisp doctor`: a checklist of everything a suggestion depends on, with a fix for each
//! failure. The checks are built from plain [`Findings`], so the wording is tested without a
//! llama-server, a daemon or a terminal.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::model::LlamaClient;
use crate::screen::{Source, Terminal};
use crate::status::{self, Report};
use crate::{paths, service, setup};

/// How one check went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// `✓`
    Ok,
    /// `✗`: suggestions do not work, or work worse than configured, until this is fixed.
    Failed,
    /// `–`: nothing is wrong, but something optional is off or absent.
    Skipped,
}

/// One line of the checklist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub outcome: Outcome,
    pub text: String,
    /// What to do about a failure, or how to turn on something skipped.
    pub hint: Option<String>,
}

impl Check {
    fn new(outcome: Outcome, text: impl Into<String>) -> Self {
        Self { outcome, text: text.into(), hint: None }
    }

    fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// What was found on this machine and in this shell.
#[derive(Debug, Clone, Default)]
pub struct Findings {
    /// `llama-server` on `PATH`.
    pub server_binary: Option<PathBuf>,
    pub llm_url: String,
    pub report: Report,
    pub zshrc: PathBuf,
    pub zshrc_loads_wisp: bool,
    pub screen_source: Option<Source>,
    /// Non-blank lines the screen source returned.
    pub screen_lines: usize,
}

impl Findings {
    /// Looks at the machine and at the terminal this process runs in.
    ///
    /// # Errors
    ///
    /// Returns an error if `HOME` is not set.
    pub async fn gather(socket: &Path, model: &LlamaClient, llm_url: &str) -> anyhow::Result<Self> {
        let terminal = Terminal::from_env();
        let zshrc = paths::zshrc()?;
        let (report, screen, zshrc_text) = tokio::join!(
            status::collect(socket, model),
            terminal.capture(),
            tokio::fs::read_to_string(&zshrc),
        );
        let path = std::env::var_os("PATH").unwrap_or_default();
        Ok(Self {
            server_binary: service::find_on_path("llama-server", &path),
            llm_url: llm_url.to_owned(),
            report,
            zshrc_loads_wisp: zshrc_text.is_ok_and(|text| loads_wisp(&text)),
            zshrc,
            screen_source: terminal.source(),
            screen_lines: screen.lines().filter(|line| !line.trim().is_empty()).count(),
        })
    }

    /// The checklist, in the order a suggestion needs things.
    #[must_use]
    pub fn checks(&self) -> Vec<Check> {
        vec![
            self.binary_check(),
            self.llama_check(),
            self.daemon_check(),
            self.pause_check(),
            self.zshrc_check(),
            self.screen_check(),
        ]
    }

    fn binary_check(&self) -> Check {
        match &self.server_binary {
            Some(path) => {
                Check::new(Outcome::Ok, format!("llama-server binary: {}", path.display()))
            }
            // A server that answers anyway is run by something else, e.g. another machine.
            None if self.report.llama.healthy => Check::new(
                Outcome::Skipped,
                "llama-server binary: not on PATH (the server runs elsewhere)",
            ),
            None => Check::new(Outcome::Failed, "llama-server binary: not found on PATH")
                .hint(setup::install_hint()),
        }
    }

    fn llama_check(&self) -> Check {
        let llama = &self.report.llama;
        if llama.healthy {
            let model = llama.model.as_deref().unwrap_or("unknown model");
            return Check::new(Outcome::Ok, format!("llama-server: {model} at {}", self.llm_url));
        }
        // The first part says what is wrong; the rest is the HTTP client's error chain.
        let reason = llama.error.as_deref().and_then(|error| error.split(": ").next());
        let reason = reason.unwrap_or("not reachable");
        let hint = if self.server_binary.is_some() {
            "run `wisp setup` to download the model and keep llama-server running"
        } else {
            "install llama-server, then run `wisp setup`"
        };
        Check::new(Outcome::Failed, format!("llama-server: down ({reason})")).hint(hint)
    }

    fn daemon_check(&self) -> Check {
        let Some(daemon) = &self.report.daemon else {
            return Check::new(
                Outcome::Skipped,
                "daemon: not running (starts with the next shell prompt)",
            );
        };
        let shells = match daemon.shells {
            1 => "1 shell".to_owned(),
            n => format!("{n} shells"),
        };
        match &daemon.last_error {
            Some(error) => {
                Check::new(Outcome::Failed, format!("daemon: {shells}, last error: {error}"))
                    .hint("the daemon could not get a suggestion; see `wisp status` and the log")
            }
            None => Check::new(Outcome::Ok, format!("daemon: running, {shells}")),
        }
    }

    fn pause_check(&self) -> Check {
        if self.report.paused {
            Check::new(Outcome::Skipped, "suggestions: paused").hint("`wisp resume` turns them on")
        } else {
            Check::new(Outcome::Ok, "suggestions: on")
        }
    }

    fn zshrc_check(&self) -> Check {
        let zshrc = self.zshrc.display();
        if self.zshrc_loads_wisp {
            Check::new(Outcome::Ok, format!("shell: {zshrc} loads wisp"))
        } else {
            Check::new(Outcome::Failed, format!("shell: {zshrc} does not load wisp")).hint(format!(
                "add this line to the end of {zshrc}, then open a new shell: \
                 eval \"$(wisp init zsh)\""
            ))
        }
    }

    fn screen_check(&self) -> Check {
        let Some(source) = self.screen_source else {
            return Check::new(
                Outcome::Skipped,
                "screen: no source for this terminal (history, files and git are still used)",
            )
            .hint("run the shell in tmux, WezTerm or kitty, or set WISP_SCREEN_CMD");
        };
        let name = source.name();
        match self.screen_lines {
            0 => Check::new(Outcome::Failed, format!("screen: {name} returned no text"))
                .hint(empty_screen_hint(source)),
            1 => Check::new(Outcome::Ok, format!("screen: {name} (captured 1 line)")),
            lines => Check::new(Outcome::Ok, format!("screen: {name} (captured {lines} lines)")),
        }
    }
}

fn empty_screen_hint(source: Source) -> &'static str {
    match source {
        Source::Custom => {
            "WISP_SCREEN_CMD must print the screen and exit 0 within 500 ms; \
             try it with: sh -c \"$WISP_SCREEN_CMD\""
        }
        Source::Tmux => "check that `tmux` is on PATH and `tmux capture-pane -p` prints text",
        Source::WezTerm => "check that `wezterm` is on PATH and `wezterm cli get-text` prints text",
        Source::Kitty => {
            "set `allow_remote_control yes` and `listen_on unix:/tmp/kitty` in kitty.conf, \
             then restart kitty"
        }
    }
}

/// Whether zshrc text runs `wisp init zsh` on a line that is not commented out.
#[must_use]
pub fn loads_wisp(zshrc: &str) -> bool {
    zshrc.lines().any(|line| {
        let code = line.split_once('#').map_or(line, |(code, _)| code);
        code.contains("wisp init zsh")
    })
}

/// How many checks failed; `wisp doctor` exits non-zero when any did.
#[must_use]
pub fn failures(checks: &[Check]) -> usize {
    checks.iter().filter(|check| check.outcome == Outcome::Failed).count()
}

/// The checklist as text: one marked line per check, with its hint indented below.
#[must_use]
pub fn render(checks: &[Check]) -> String {
    let mut text = String::new();
    for check in checks {
        let (mark, lead) = match check.outcome {
            Outcome::Ok => ('✓', "note"),
            Outcome::Failed => ('✗', "fix"),
            Outcome::Skipped => ('–', "tip"),
        };
        let _ = writeln!(text, "{mark} {}", check.text);
        if let Some(hint) = &check.hint {
            let _ = writeln!(text, "    {lead}: {hint}");
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::LlamaStatus;
    use crate::stats::DaemonStats;

    fn healthy() -> Findings {
        Findings {
            server_binary: Some("/opt/homebrew/bin/llama-server".into()),
            llm_url: "http://127.0.0.1:8012".into(),
            report: Report {
                paused: false,
                daemon: Some(DaemonStats { shells: 2, ..DaemonStats::default() }),
                llama: LlamaStatus {
                    healthy: true,
                    model: Some("Qwen2.5-Coder-1.5B-Q8_0".into()),
                    error: None,
                },
                llama_memory_mb: None,
            },
            zshrc: "/Users/me/.zshrc".into(),
            zshrc_loads_wisp: true,
            screen_source: Some(Source::Tmux),
            screen_lines: 12,
        }
    }

    #[test]
    fn a_healthy_setup_passes_every_check() {
        let checks = healthy().checks();
        assert_eq!(failures(&checks), 0);
        assert_eq!(
            render(&checks),
            "✓ llama-server binary: /opt/homebrew/bin/llama-server\n\
             ✓ llama-server: Qwen2.5-Coder-1.5B-Q8_0 at http://127.0.0.1:8012\n\
             ✓ daemon: running, 2 shells\n\
             ✓ suggestions: on\n\
             ✓ shell: /Users/me/.zshrc loads wisp\n\
             ✓ screen: tmux (captured 12 lines)\n"
        );
    }

    #[test]
    fn a_fresh_machine_fails_with_a_fix_for_each_problem() {
        let findings = Findings {
            llm_url: "http://127.0.0.1:8012".into(),
            report: Report {
                llama: LlamaStatus {
                    healthy: false,
                    model: None,
                    error: Some(
                        "no llama-server at http://127.0.0.1:8012: error sending request".into(),
                    ),
                },
                ..Report::default()
            },
            zshrc: "/home/me/.zshrc".into(),
            ..Findings::default()
        };
        let checks = findings.checks();
        assert_eq!(failures(&checks), 3);
        let text = render(&checks);
        for expected in [
            "✗ llama-server binary: not found on PATH\n    fix: install it with",
            "✗ llama-server: down (no llama-server at http://127.0.0.1:8012)\n    \
             fix: install llama-server, then run `wisp setup`\n",
            "– daemon: not running (starts with the next shell prompt)\n",
            "✗ shell: /home/me/.zshrc does not load wisp\n    fix: add this line to the end of \
             /home/me/.zshrc, then open a new shell: eval \"$(wisp init zsh)\"\n",
            "– screen: no source for this terminal (history, files and git are still used)\n    \
             tip: run the shell in tmux, WezTerm or kitty, or set WISP_SCREEN_CMD\n",
        ] {
            assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
        }
    }

    #[test]
    fn a_stopped_server_with_the_binary_installed_points_at_setup() {
        let mut findings = healthy();
        findings.report.llama = LlamaStatus::default();
        let check = findings.llama_check();
        assert_eq!(check.outcome, Outcome::Failed);
        assert_eq!(check.text, "llama-server: down (not reachable)");
        assert!(check.hint.unwrap_or_default().starts_with("run `wisp setup`"));
    }

    #[test]
    fn a_remote_server_does_not_need_the_binary() {
        let findings = Findings { server_binary: None, ..healthy() };
        assert_eq!(findings.binary_check().outcome, Outcome::Skipped);
        assert_eq!(failures(&findings.checks()), 0);
    }

    #[test]
    fn pause_is_reported_but_is_not_a_failure() {
        let mut findings = healthy();
        findings.report.paused = true;
        let check = findings.pause_check();
        assert_eq!(check.outcome, Outcome::Skipped);
        assert_eq!(
            render(&[check]),
            "– suggestions: paused\n    tip: `wisp resume` turns them on\n"
        );
    }

    #[test]
    fn a_daemon_error_is_a_failure() {
        let mut findings = healthy();
        if let Some(daemon) = findings.report.daemon.as_mut() {
            daemon.shells = 1;
            daemon.last_error = Some("timeout".into());
        }
        let check = findings.daemon_check();
        assert_eq!(check.outcome, Outcome::Failed);
        assert_eq!(check.text, "daemon: 1 shell, last error: timeout");
    }

    #[test]
    fn a_source_that_returns_nothing_fails_with_a_hint_for_that_source() {
        let findings =
            Findings { screen_source: Some(Source::Kitty), screen_lines: 0, ..healthy() };
        let check = findings.screen_check();
        assert_eq!(check.outcome, Outcome::Failed);
        assert_eq!(check.text, "screen: kitty (experimental) returned no text");
        assert!(check.hint.unwrap_or_default().contains("allow_remote_control"));

        let one_line = Findings { screen_lines: 1, ..healthy() };
        assert_eq!(one_line.screen_check().text, "screen: tmux (captured 1 line)");
    }

    #[test]
    fn finds_the_init_line_unless_it_is_commented_out() {
        assert!(loads_wisp("export A=1\neval \"$(wisp init zsh)\"\n"));
        assert!(loads_wisp("  source <(/opt/bin/wisp init zsh) # suggestions\n"));
        assert!(!loads_wisp("# eval \"$(wisp init zsh)\"\n"));
        assert!(!loads_wisp("eval \"$(starship init zsh)\"\n"));
        assert!(!loads_wisp(""));
    }
}
