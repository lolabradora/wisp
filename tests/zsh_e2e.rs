//! Runs `tests/zsh_e2e.sh`: a real zsh in tmux with the plugin, the daemon and a fake
//! llama-server. The script skips itself (and passes) when tmux, zsh or python3 is missing,
//! unless `WISP_E2E_REQUIRE` is set.

use std::process::Command;

#[test]
fn ghost_text_is_shown_accepted_and_run_in_a_real_zsh() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/zsh_e2e.sh");
    // Output is inherited, so the script's own failure report shows up in the test log.
    let status =
        Command::new("bash").arg(script).env("WISP_BIN", env!("CARGO_BIN_EXE_wisp")).status();
    let status = status.unwrap();
    assert!(status.success(), "{script} failed with {status}");
}
