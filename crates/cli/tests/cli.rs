//! End-to-end tests over the built binary.
//!
//! These exercise the wiring the unit tests cannot: argument parsing, the
//! composition root, rendering, and exit codes.

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

fn toolbox() -> Command {
    Command::cargo_bin("toolbox").expect("the binary is built")
}

#[test]
fn shows_help_without_arguments() {
    toolbox()
        .assert()
        .failure()
        .stderr(contains("Usage").and(contains("accounts")));
}

#[test]
fn every_area_is_reachable_from_help() {
    let output = toolbox().arg("--help").assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).expect("utf-8");

    for area in ["accounts", "characters", "items", "spawns", "server"] {
        assert!(stdout.contains(area), "help should mention {area}");
    }
}

#[test]
fn bans_the_seeded_account() {
    toolbox()
        .args(["accounts", "ban", "player01", "--reason", "botting"])
        .assert()
        .success()
        .stdout(contains("Banned player01 permanently"));
}

#[test]
fn bans_for_a_fixed_number_of_days() {
    toolbox()
        .args([
            "accounts", "ban", "player01", "--reason", "botting", "--days", "7",
        ])
        .assert()
        .success()
        .stdout(contains("for 7 day(s)"));
}

#[test]
fn renders_json_when_asked() {
    let output = toolbox()
        .args([
            "--output", "json", "accounts", "ban", "player01", "--reason", "botting",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).expect("utf-8");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");

    assert_eq!(parsed["account_name"], "player01");
    assert_eq!(parsed["permanent"], true);
    assert_eq!(parsed["was_already_banned"], false);
}

#[test]
fn a_missing_account_exits_with_code_three() {
    toolbox()
        .args(["accounts", "ban", "nobody99", "--reason", "botting"])
        .assert()
        .code(3)
        .stderr(contains("not found"));
}

#[test]
fn invalid_input_exits_with_code_two() {
    toolbox()
        .args(["accounts", "ban", "ab", "--reason", "botting"])
        .assert()
        .code(2);
}

#[test]
fn an_unimplemented_command_exits_with_code_four() {
    toolbox()
        .args(["items", "show", "ITEM_CH_SWORD_01_A"])
        .assert()
        .code(4)
        .stderr(contains("not implemented yet"));
}

#[test]
fn generates_shell_completions() {
    toolbox()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(contains("toolbox"));
}
