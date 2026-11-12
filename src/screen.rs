//! Reads the text on the terminal screen, so the model sees the output the user is looking at.
//!
//! No terminal offers this the same way, so there are several sources and the first one the
//! shell's environment points at wins (see [`Terminal::capture_command`]). The zsh plugin sends
//! that environment with every finished command, because the daemon serves many shells and its
//! own environment says nothing about theirs.

use serde::Deserialize;

use crate::context::run_probe;

/// How many lines of screen and scrollback are asked for.
pub(crate) const SCREEN_LINES: usize = 60;

/// What a shell knows about the terminal it runs in: the `term` object of a `done` message.
/// A field is empty when the shell's environment does not have the variable.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Terminal {
    /// `$TMUX`: `<socket path>,<server pid>,<session>`.
    pub tmux: String,
    /// `$TMUX_PANE`, e.g. `%3`.
    pub tmux_pane: String,
    /// `$WEZTERM_PANE`.
    pub wezterm_pane: String,
    /// `$KITTY_WINDOW_ID`.
    pub kitty_window: String,
    /// `$KITTY_LISTEN_ON`, set when kitty.conf has `listen_on`.
    pub kitty_listen: String,
    /// `$WISP_SCREEN_CMD`: a shell command that prints the screen.
    pub screen_cmd: String,
}

/// A way of reading the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Custom,
    Tmux,
    WezTerm,
    Kitty,
}

impl Source {
    /// The name shown by `wisp doctor` and `wisp complete`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Custom => "WISP_SCREEN_CMD",
            Self::Tmux => "tmux",
            Self::WezTerm => "WezTerm",
            Self::Kitty => "kitty (experimental)",
        }
    }
}

/// The command that prints the screen for one [`Source`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureCommand {
    pub source: Source,
    pub program: &'static str,
    pub args: Vec<String>,
}

impl Terminal {
    /// The terminal of the current process, for commands run by hand such as `wisp doctor`.
    #[must_use]
    pub fn from_env() -> Self {
        let var = |name: &str| std::env::var(name).unwrap_or_default();
        Self {
            tmux: var("TMUX"),
            tmux_pane: var("TMUX_PANE"),
            wezterm_pane: var("WEZTERM_PANE"),
            kitty_window: var("KITTY_WINDOW_ID"),
            kitty_listen: var("KITTY_LISTEN_ON"),
            screen_cmd: var("WISP_SCREEN_CMD"),
        }
    }

    /// The terminal of a shell whose plugin predates the `term` object and sends only the
    /// `WezTerm` pane id.
    #[must_use]
    pub fn from_wezterm_pane(pane: String) -> Self {
        Self { wezterm_pane: pane, ..Self::default() }
    }

    /// The source that will be used, if the shell's environment points at one.
    #[must_use]
    pub fn source(&self) -> Option<Source> {
        self.capture_command().map(|command| command.source)
    }

    /// Picks the first source that applies. The user's own command wins, since it is an explicit
    /// choice. tmux comes before the emulators because inside tmux the emulator's variables are
    /// still set, but its screen may show another tmux window than the one this shell is in.
    #[must_use]
    pub fn capture_command(&self) -> Option<CaptureCommand> {
        let start_line = format!("-{SCREEN_LINES}");
        let command = |source, program, args: &[&str]| {
            Some(CaptureCommand { source, program, args: args.iter().map(|&a| a.into()).collect() })
        };
        if !self.screen_cmd.trim().is_empty() {
            return command(Source::Custom, "sh", &["-c", &self.screen_cmd]);
        }
        // The pane has to be named: the daemon is not a tmux client, so there is no current pane.
        let tmux_socket = self.tmux.split(',').next().unwrap_or_default();
        if !tmux_socket.is_empty() && !self.tmux_pane.is_empty() {
            // -J joins wrapped lines, so a long command reads as one line again.
            let pane = self.tmux_pane.as_str();
            let args =
                ["-S", tmux_socket, "capture-pane", "-p", "-J", "-t", pane, "-S", &start_line];
            return command(Source::Tmux, "tmux", &args);
        }
        if !self.wezterm_pane.is_empty() {
            let pane = self.wezterm_pane.as_str();
            let args = ["cli", "get-text", "--pane-id", pane, "--start-line", &start_line];
            return command(Source::WezTerm, "wezterm", &args);
        }
        // Without `listen_on` in kitty.conf there is no socket the daemon could reach.
        if !self.kitty_window.is_empty() && !self.kitty_listen.is_empty() {
            let window = format!("id:{}", self.kitty_window);
            let listen = self.kitty_listen.as_str();
            let args = ["@", "--to", listen, "get-text", "--match", &window, "--extent", "screen"];
            return command(Source::Kitty, "kitty", &args);
        }
        None
    }

    /// The screen text, or an empty string when there is no source or it fails; suggestions
    /// then rely on history, files and git alone.
    pub async fn capture(&self) -> String {
        match self.capture_command() {
            Some(command) => run_probe(command.program, &command.args).await,
            None => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn everything() -> Terminal {
        Terminal {
            tmux: "/private/tmp/tmux-501/default,4242,0".into(),
            tmux_pane: "%3".into(),
            wezterm_pane: "7".into(),
            kitty_window: "2".into(),
            kitty_listen: "unix:/tmp/kitty".into(),
            screen_cmd: "my-screen --plain".into(),
        }
    }

    fn command_line(terminal: &Terminal) -> String {
        let command = terminal.capture_command().unwrap();
        format!("{} {}", command.program, command.args.join(" "))
    }

    #[test]
    fn sources_are_tried_in_order() {
        let mut terminal = everything();
        assert_eq!(terminal.source(), Some(Source::Custom));
        terminal.screen_cmd = "  ".into();
        assert_eq!(terminal.source(), Some(Source::Tmux));
        terminal.tmux.clear();
        assert_eq!(terminal.source(), Some(Source::WezTerm));
        terminal.wezterm_pane.clear();
        assert_eq!(terminal.source(), Some(Source::Kitty));
        terminal.kitty_listen.clear();
        assert_eq!(terminal.source(), None);
        assert_eq!(Terminal::default().capture_command(), None);
    }

    #[test]
    fn the_custom_command_runs_through_sh() {
        let command = everything().capture_command().unwrap();
        assert_eq!(command.program, "sh");
        assert_eq!(command.args, ["-c", "my-screen --plain"]);
    }

    #[test]
    fn tmux_is_asked_through_the_socket_of_the_shell() {
        let terminal = Terminal { screen_cmd: String::new(), ..everything() };
        assert_eq!(
            command_line(&terminal),
            "tmux -S /private/tmp/tmux-501/default capture-pane -p -J -t %3 -S -60"
        );
    }

    #[test]
    fn tmux_needs_both_a_socket_and_a_pane() {
        let no_pane = Terminal { tmux: "/tmp/tmux-1/default,1,0".into(), ..Terminal::default() };
        assert_eq!(no_pane.source(), None);
        let no_socket =
            Terminal { tmux: ",1,0".into(), tmux_pane: "%0".into(), ..Terminal::default() };
        assert_eq!(no_socket.source(), None);
    }

    #[test]
    fn wezterm_is_asked_for_the_pane() {
        let terminal = Terminal::from_wezterm_pane("7".into());
        assert_eq!(command_line(&terminal), "wezterm cli get-text --pane-id 7 --start-line -60");
    }

    #[test]
    fn kitty_is_asked_over_its_remote_control_socket() {
        let terminal = Terminal {
            kitty_window: "2".into(),
            kitty_listen: "unix:/tmp/kitty".into(),
            ..Terminal::default()
        };
        assert_eq!(
            command_line(&terminal),
            "kitty @ --to unix:/tmp/kitty get-text --match id:2 --extent screen"
        );
    }

    #[test]
    fn kitty_without_a_listen_socket_is_no_source() {
        let terminal = Terminal { kitty_window: "2".into(), ..Terminal::default() };
        assert_eq!(terminal.source(), None);
    }

    #[tokio::test]
    async fn captures_the_output_of_the_custom_command() {
        let terminal =
            Terminal { screen_cmd: "printf 'one\\ntwo\\n'".into(), ..Terminal::default() };
        assert_eq!(terminal.capture().await, "one\ntwo\n");
    }

    #[tokio::test]
    async fn a_failing_or_missing_source_captures_nothing() {
        let failing = Terminal { screen_cmd: "echo partial; exit 3".into(), ..Terminal::default() };
        assert_eq!(failing.capture().await, "");
        assert_eq!(Terminal::default().capture().await, "");
    }
}
