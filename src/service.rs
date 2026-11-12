//! What `wisp setup` installs, as text: the launchd agents and the systemd unit that keep
//! llama-server (and the menu bar item) running. Nothing here touches the system; `setup` does.

use std::ffi::{OsStr, OsString};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::paths::{LLAMA_AGENT, MENUBAR_AGENT};

const LAUNCHD_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
"#;

/// Environment variables that move llama-server's model cache. A service does not inherit the
/// shell's environment, so they are copied into it; otherwise it would look for the weights in
/// the default place and, being offline, fail to start.
pub const CACHE_VARS: [&str; 3] = ["LLAMA_CACHE", "HF_HUB_CACHE", "HF_HOME"];

/// Which Qwen2.5-Coder model llama-server loads.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ModelSize {
    /// Fastest; about 2 GB of memory.
    #[default]
    #[value(name = "1.5b")]
    Small,
    /// Better suggestions for about twice the memory and latency.
    #[value(name = "3b")]
    Large,
}

impl ModelSize {
    /// The size as `--model` spells it.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "1.5b",
            Self::Large => "3b",
        }
    }

    /// The llama-server flag that selects this model and downloads it when missing.
    #[must_use]
    pub fn preset(self) -> String {
        format!("--fim-qwen-{}-default", self.label())
    }

    /// The Hugging Face repository (under `ggml-org`) the preset downloads from.
    #[must_use]
    pub fn repo(self) -> &'static str {
        match self {
            Self::Small => "Qwen2.5-Coder-1.5B-Q8_0-GGUF",
            Self::Large => "Qwen2.5-Coder-3B-Q8_0-GGUF",
        }
    }
}

/// The llama-server that a background service keeps running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlamaService {
    /// Absolute path of the `llama-server` binary; services do not search `PATH`.
    pub server: PathBuf,
    pub model: ModelSize,
    pub port: u16,
    /// Extra environment for the service, see [`CACHE_VARS`].
    pub env: Vec<(String, String)>,
}

impl LlamaService {
    /// The arguments after the program name.
    #[must_use]
    pub fn args(&self) -> Vec<String> {
        let port = self.port.to_string();
        let preset = self.model.preset();
        [&preset, "--offline", "--ctx-size", "32768", "--parallel", "4", "--host", "127.0.0.1"]
            .into_iter()
            .chain(["--port", &port])
            .map(str::to_owned)
            .collect()
    }

    /// The launchd agent for macOS, writing llama-server's output to `log`.
    #[must_use]
    pub fn launchd_plist(&self, log: &Path) -> String {
        let mut plist = launchd_start(LLAMA_AGENT);
        for arg in [self.server.display().to_string(), self.model.preset()] {
            push_string(&mut plist, 4, &arg);
        }
        plist.push_str(
            "    <!-- Offline, so a missing network at login cannot stop the agent. `wisp setup`
         downloads the weights before it installs this file. -->
    <string>--offline</string>
    <!-- The preset reserves 4 x 32k tokens of KV cache (~4.7 GB for 3B). wisp prompts stay under
         ~5k tokens, so 4 slots x 8k keeps four shells cached for a quarter of the memory. -->
    <string>--ctx-size</string>
    <string>32768</string>
    <string>--parallel</string>
    <string>4</string>
    <string>--host</string>
    <string>127.0.0.1</string>
    <string>--port</string>
",
        );
        push_string(&mut plist, 4, &self.port.to_string());
        plist.push_str("  </array>\n");
        if !self.env.is_empty() {
            plist.push_str("  <key>EnvironmentVariables</key>\n  <dict>\n");
            for (name, value) in &self.env {
                let _ = writeln!(plist, "    <key>{}</key>", xml_escape(name));
                push_string(&mut plist, 4, value);
            }
            plist.push_str("  </dict>\n");
        }
        plist.push_str(
            "  <key>RunAtLoad</key>
  <true/>
  <!-- Restarts llama-server whenever it exits, so a crash costs one suggestion. -->
  <key>KeepAlive</key>
  <true/>
  <!-- Interactive keeps macOS from throttling it onto efficiency cores; latency matters here. -->
",
        );
        launchd_end(&mut plist, log);
        plist
    }

    /// The systemd user unit for Linux. Its output goes to the journal.
    #[must_use]
    pub fn systemd_unit(&self) -> String {
        let server = self.server.display().to_string();
        let command: Vec<String> = std::iter::once(server)
            .chain(self.args())
            .map(|word| systemd_quote(&word, "$$"))
            .collect();
        let mut unit = format!(
            "# Installed by `wisp setup`; running it again replaces this file.
[Unit]
Description=llama-server for wisp shell suggestions

[Service]
# --offline: a missing network at login cannot stop the service. `wisp setup` downloads the
# weights before it installs this file.
# --ctx-size 32768 --parallel 4: the preset reserves 4 x 32k tokens of KV cache (~4.7 GB for
# 3B). wisp prompts stay under ~5k tokens, so 4 slots x 8k keeps four shells cached for a
# quarter of the memory.
ExecStart={}
",
            command.join(" ")
        );
        for (name, value) in &self.env {
            let _ =
                writeln!(unit, "Environment={}", systemd_quote(&format!("{name}={value}"), "$"));
        }
        unit.push_str(
            "# Restarts llama-server whenever it exits, so a crash costs one suggestion.
Restart=always
RestartSec=1

[Install]
WantedBy=default.target
",
        );
        unit
    }
}

/// The launchd agent that shows `wisp menubar` at login. Choosing Quit in the menu stops it
/// until the next login (a successful exit), while a crash restarts it.
#[must_use]
pub fn menubar_plist(wisp: &Path, log: &Path) -> String {
    let mut plist = launchd_start(MENUBAR_AGENT);
    push_string(&mut plist, 4, &wisp.display().to_string());
    plist.push_str(
        "    <string>menubar</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <dict>
    <key>SuccessfulExit</key>
    <false/>
  </dict>
",
    );
    launchd_end(&mut plist, log);
    plist
}

/// Everything up to the first program argument.
fn launchd_start(label: &str) -> String {
    let mut plist = String::from(LAUNCHD_HEADER);
    plist.push_str("  <key>Label</key>\n");
    push_string(&mut plist, 2, label);
    plist.push_str("  <key>ProgramArguments</key>\n  <array>\n");
    plist
}

/// The keys both agents end with.
fn launchd_end(plist: &mut String, log: &Path) {
    let log = log.display().to_string();
    plist.push_str("  <key>ProcessType</key>\n  <string>Interactive</string>\n");
    for key in ["StandardOutPath", "StandardErrorPath"] {
        let _ = writeln!(plist, "  <key>{key}</key>");
        push_string(plist, 2, &log);
    }
    plist.push_str("</dict>\n</plist>\n");
}

fn push_string(plist: &mut String, indent: usize, value: &str) {
    let _ = writeln!(plist, "{:indent$}<string>{}</string>", "", xml_escape(value));
}

fn xml_escape(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Quotes one word of a unit file setting. systemd expands `%` specifiers everywhere and `$`
/// variables in `ExecStart`, so a path containing either must be escaped to stay literal;
/// `dollar` is what a literal `$` is written as in the setting at hand.
fn systemd_quote(word: &str, dollar: &str) -> String {
    let plain = |c: char| c.is_ascii_alphanumeric() || "_-./=:+,@".contains(c);
    if !word.is_empty() && word.chars().all(plain) {
        return word.to_owned();
    }
    let escaped =
        word.replace('\\', r"\\").replace('"', r#"\""#).replace('%', "%%").replace('$', dollar);
    format!("\"{escaped}\"")
}

/// The first executable called `name` in the directories of `path` (the `PATH` variable).
#[must_use]
pub fn find_on_path(name: &str, path: &OsStr) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt as _;
    std::env::split_paths(path).map(|dir| dir.join(name)).find(|candidate| {
        std::fs::metadata(candidate)
            .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
    })
}

/// The directories llama-server may keep downloaded models in, given a lookup for environment
/// variables: the ones that override the cache location, then the Hugging Face default.
#[must_use]
pub fn hub_dirs(env: impl Fn(&str) -> Option<OsString>, home: &Path) -> Vec<PathBuf> {
    let var = |name: &str| env(name).filter(|value| !value.is_empty()).map(PathBuf::from);
    [
        var("LLAMA_CACHE"),
        var("HF_HUB_CACHE"),
        var("HF_HOME").map(|dir| dir.join("hub")),
        Some(home.join(".cache/huggingface/hub")),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Whether the weights of `model` are in one of `hub_dirs`, as
/// `models--ggml-org--<repo>/snapshots/<revision>/<file>.gguf`.
///
/// A wrong "no" only costs a moment: `wisp setup` then starts llama-server once, which finds
/// the weights itself and is ready right away.
#[must_use]
pub fn model_is_cached(hub_dirs: &[PathBuf], model: ModelSize) -> bool {
    let entries = |dir: PathBuf| std::fs::read_dir(dir).into_iter().flatten().flatten();
    hub_dirs
        .iter()
        .map(|hub| hub.join(format!("models--ggml-org--{}", model.repo())).join("snapshots"))
        .flat_map(entries)
        .flat_map(|revision| entries(revision.path()))
        .map(|file| file.path())
        // Snapshot entries are symlinks into `blobs/`; `exists` rejects dangling ones.
        .any(|file| file.extension().is_some_and(|ext| ext == "gguf") && file.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> LlamaService {
        LlamaService {
            server: "/opt/homebrew/bin/llama-server".into(),
            model: ModelSize::Small,
            port: 8012,
            env: Vec::new(),
        }
    }

    /// The `<string>` values of the plist's `ProgramArguments`.
    fn program_arguments(plist: &str) -> Vec<&str> {
        let array = plist.split_once("<array>").unwrap().1.split_once("</array>").unwrap().0;
        array
            .lines()
            .filter_map(|line| line.trim().strip_prefix("<string>")?.strip_suffix("</string>"))
            .collect()
    }

    #[test]
    fn model_sizes_map_to_presets_and_repos() {
        assert_eq!(ModelSize::Small.preset(), "--fim-qwen-1.5b-default");
        assert_eq!(ModelSize::Large.preset(), "--fim-qwen-3b-default");
        assert_eq!(ModelSize::Small.repo(), "Qwen2.5-Coder-1.5B-Q8_0-GGUF");
        assert_eq!(ModelSize::Large.repo(), "Qwen2.5-Coder-3B-Q8_0-GGUF");
    }

    #[test]
    fn llama_server_runs_offline_on_loopback_with_four_small_slots() {
        let args = LlamaService { model: ModelSize::Large, port: 9000, ..service() }.args();
        assert_eq!(
            args.join(" "),
            "--fim-qwen-3b-default --offline --ctx-size 32768 --parallel 4 --host 127.0.0.1 \
             --port 9000"
        );
    }

    #[test]
    fn launchd_plist_runs_the_same_command_as_args() {
        let service = service();
        let plist =
            service.launchd_plist(Path::new("/Users/me/Library/Logs/wisp-llama-server.log"));
        let mut expected = vec!["/opt/homebrew/bin/llama-server".to_owned()];
        expected.extend(service.args());
        assert_eq!(program_arguments(&plist), expected);
    }

    #[test]
    fn launchd_plist_keeps_the_server_alive_and_interactive() {
        let plist = service().launchd_plist(Path::new("/logs/llama.log"));
        assert!(plist.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist"));
        assert!(plist.ends_with("</dict>\n</plist>\n"));
        for expected in [
            "  <key>Label</key>\n  <string>dev.wisp.llama-server</string>\n",
            "  <key>RunAtLoad</key>\n  <true/>\n",
            "  <key>KeepAlive</key>\n  <true/>\n",
            "  <key>ProcessType</key>\n  <string>Interactive</string>\n",
            "  <key>StandardOutPath</key>\n  <string>/logs/llama.log</string>\n",
            "  <key>StandardErrorPath</key>\n  <string>/logs/llama.log</string>\n",
        ] {
            assert!(plist.contains(expected), "missing {expected:?} in:\n{plist}");
        }
        assert!(!plist.contains("EnvironmentVariables"), "{plist}");
    }

    #[test]
    fn launchd_plist_escapes_paths_and_carries_the_cache_location() {
        let service = LlamaService {
            server: "/opt/a&b/llama-server".into(),
            env: vec![("HF_HOME".into(), "/Volumes/<big>/hf".into())],
            ..service()
        };
        let plist = service.launchd_plist(Path::new("/logs/llama.log"));
        assert!(plist.contains("    <string>/opt/a&amp;b/llama-server</string>\n"), "{plist}");
        assert!(
            plist.contains(
                "  <key>EnvironmentVariables</key>\n  <dict>\n    <key>HF_HOME</key>\n    \
                 <string>/Volumes/&lt;big&gt;/hf</string>\n  </dict>\n"
            ),
            "{plist}"
        );
    }

    #[test]
    fn menubar_plist_restarts_only_after_a_crash() {
        let plist = menubar_plist(Path::new("/usr/local/bin/wisp"), Path::new("/logs/menubar.log"));
        assert_eq!(program_arguments(&plist), ["/usr/local/bin/wisp", "menubar"]);
        for expected in [
            "  <string>dev.wisp.menubar</string>\n",
            "  <key>KeepAlive</key>\n  <dict>\n    <key>SuccessfulExit</key>\n    <false/>\n  </dict>\n",
            "  <key>ProcessType</key>\n  <string>Interactive</string>\n",
            "  <key>StandardErrorPath</key>\n  <string>/logs/menubar.log</string>\n",
        ] {
            assert!(plist.contains(expected), "missing {expected:?} in:\n{plist}");
        }
    }

    #[test]
    fn systemd_unit_restarts_the_server_and_starts_at_login() {
        let unit = service().systemd_unit();
        for expected in [
            "[Unit]\nDescription=llama-server for wisp shell suggestions\n",
            "\nExecStart=/opt/homebrew/bin/llama-server --fim-qwen-1.5b-default --offline \
             --ctx-size 32768 --parallel 4 --host 127.0.0.1 --port 8012\n",
            "\nRestart=always\n",
            "\n[Install]\nWantedBy=default.target\n",
        ] {
            assert!(unit.contains(expected), "missing {expected:?} in:\n{unit}");
        }
        assert!(!unit.contains("Environment="), "{unit}");
    }

    #[test]
    fn systemd_unit_quotes_awkward_paths() {
        let service = LlamaService {
            server: "/home/me/my tools/100%/$x/llama-server".into(),
            env: vec![("HF_HOME".into(), "/mnt/big disk/$hf".into())],
            ..service()
        };
        let unit = service.systemd_unit();
        assert!(
            unit.contains("\nExecStart=\"/home/me/my tools/100%%/$$x/llama-server\" --fim-qwen"),
            "{unit}"
        );
        assert!(unit.contains("\nEnvironment=\"HF_HOME=/mnt/big disk/$hf\"\n"), "{unit}");
    }

    #[test]
    fn finds_only_executables_on_the_path() {
        use std::os::unix::fs::PermissionsExt as _;
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let data = first.path().join("llama-server");
        let binary = second.path().join("llama-server");
        std::fs::write(&data, "").unwrap();
        std::fs::write(&binary, "").unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();
        let path = std::env::join_paths([first.path(), second.path()]).unwrap();
        assert_eq!(find_on_path("llama-server", &path), Some(binary));
        assert_eq!(find_on_path("no-such-tool", &path), None);
    }

    #[test]
    fn cache_overrides_come_before_the_default_hub() {
        let env = |name: &str| match name {
            "HF_HOME" => Some(OsString::from("/data/hf")),
            "LLAMA_CACHE" => Some(OsString::new()),
            _ => None,
        };
        assert_eq!(
            hub_dirs(env, Path::new("/home/me")),
            [Path::new("/data/hf/hub"), Path::new("/home/me/.cache/huggingface/hub")]
        );
    }

    #[test]
    fn finds_downloaded_weights_of_the_right_size_only() {
        let hub = tempfile::tempdir().unwrap();
        let hubs = [PathBuf::from("/nonexistent/hub"), hub.path().to_owned()];
        assert!(!model_is_cached(&hubs, ModelSize::Small));

        let snapshot =
            hub.path().join("models--ggml-org--Qwen2.5-Coder-1.5B-Q8_0-GGUF/snapshots/abc123");
        std::fs::create_dir_all(&snapshot).unwrap();
        std::fs::write(snapshot.join("README.md"), "").unwrap();
        assert!(!model_is_cached(&hubs, ModelSize::Small));

        std::fs::write(snapshot.join("qwen2.5-coder-1.5b-q8_0.gguf"), "").unwrap();
        assert!(model_is_cached(&hubs, ModelSize::Small));
        assert!(!model_is_cached(&hubs, ModelSize::Large));
    }
}
