//! End-to-end CLI test. Shells out to the built `loust-llm-mempipe` binary
//! with the real ChatGPT fixture and verifies:
//! - exit code 0
//! - expected files written
//! - stats line on stderr contains expected fields
//! - dry-run doesn't write anything
//! - unknown adapter / format produce non-zero exit

use std::path::Path;
use std::process::Command;

/// Returns the absolute path to the test binary, resolved by Cargo at
/// test time. This is the standard way to invoke a crate's own binary
/// in integration tests.
fn bin() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_loust-llm-mempipe"))
}

const FIXTURE: &str = "tests/fixtures/chatgpt-tiny.json";

fn run(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(bin())
        .args(args)
        .env("RUST_BACKTRACE", "0")
        .output()
        .expect("failed to invoke binary");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, stdout, stderr)
}

#[test]
fn cli_version_flag_works() {
    let (code, stdout, _) = run(&["--version"]);
    assert_eq!(code, 0, "--version should exit 0");
    assert!(stdout.starts_with("loust-llm-mempipe "), "got: {}", stdout);
}

#[test]
fn cli_help_flag_works() {
    let (code, stdout, _) = run(&["--help"]);
    assert_eq!(code, 0, "--help should exit 0");
    // clap 4 writes --help output to stdout, not stderr.
    let combined = format!("{}{}", stdout, ""); // explicit: stdout
    assert!(
        combined.contains("--input"),
        "--help should list --input (got: {}...)",
        &stdout[..stdout.len().min(200)]
    );
    assert!(combined.contains("--output"), "--help should list --output");
    assert!(combined.contains("--format"), "--help should list --format");
}

#[test]
fn cli_info_flag_prints_build_metadata() {
    // --info is an early-exit that prints build metadata. It still
    // needs the required --input / --output so clap's parser doesn't
    // reject the call; we pass dummy paths since --info bails before
    // touching them.
    let (code, stdout, _) = run(&["--info", "--input", "dummy.json", "--output", "dummy/"]);
    assert_eq!(code, 0, "--info should exit 0");
    assert!(stdout.contains("loust-llm-mempipe"));
    assert!(stdout.contains("repository:"));
    assert!(stdout.contains("license:"));
}

#[test]
fn cli_full_run_chatgpt_fixture_writes_both_formats() {
    let tmp = tempfile::tempdir().unwrap();
    let out_dir = tmp.path();
    let (code, _, stderr) = run(&[
        "--input",
        FIXTURE,
        "--output",
        out_dir.to_str().unwrap(),
        "--format",
        "both",
        "--stats",
    ]);
    assert_eq!(code, 0, "run should succeed; stderr={}", stderr);

    // JSONL written
    let jsonl = out_dir.join("memory.jsonl");
    assert!(jsonl.is_file(), "memory.jsonl should exist");
    let body = std::fs::read_to_string(&jsonl).unwrap();
    let lines: Vec<&str> = body.lines().collect();
    assert!(lines.len() >= 5, "expected >=5 lines, got {}", lines.len());
    for line in &lines {
        let parsed: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("invalid JSONL line: {} -- {}", line, e));
        assert!(parsed["id"].is_string());
        assert!(parsed["source"].is_string());
        assert!(parsed["role"].is_string());
    }

    // Markdown written under project subdirs
    let mut md_count = 0;
    for entry in std::fs::read_dir(out_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().join("").is_dir() {
            for sub in std::fs::read_dir(entry.path()).unwrap() {
                let sub = sub.unwrap();
                if sub.path().extension().and_then(|s| s.to_str()) == Some("md") {
                    md_count += 1;
                }
            }
        }
    }
    assert!(md_count >= 1, "expected at least 1 MD file");

    // Stats line on stderr
    assert!(
        stderr.contains("stats:"),
        "stderr should contain stats: {}",
        stderr
    );
    assert!(stderr.contains("in="));
    assert!(stderr.contains("out="));
    assert!(stderr.contains("detected adapter: chatgpt"));
}

#[test]
fn cli_dry_run_writes_nothing_but_prints_stats() {
    let tmp = tempfile::tempdir().unwrap();
    let out_dir = tmp.path();
    let (code, _, stderr) = run(&[
        "--input",
        FIXTURE,
        "--output",
        out_dir.to_str().unwrap(),
        "--format",
        "both",
        "--stats",
        "--dry-run",
    ]);
    assert_eq!(code, 0, "dry-run should exit 0");
    assert!(stderr.contains("dry run: nothing written"));
    // No files in the output dir
    let entries: Vec<_> = std::fs::read_dir(out_dir).unwrap().collect();
    assert!(entries.is_empty(), "dry run should not write anything");
}

#[test]
fn cli_with_explicit_adapter_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    let (code, _, stderr) = run(&[
        "--input",
        FIXTURE,
        "--output",
        tmp.path().to_str().unwrap(),
        "--adapter",
        "chatgpt",
    ]);
    assert_eq!(
        code, 0,
        "explicit adapter should succeed; stderr={}",
        stderr
    );
    assert!(tmp.path().join("memory.jsonl").is_file());
}

#[test]
fn cli_rejects_unknown_format() {
    let tmp = tempfile::tempdir().unwrap();
    let (code, _, _) = run(&[
        "--input",
        FIXTURE,
        "--output",
        tmp.path().to_str().unwrap(),
        "--format",
        "xml",
    ]);
    assert_ne!(code, 0, "invalid format should fail");
}

#[test]
fn cli_rejects_unknown_adapter() {
    let tmp = tempfile::tempdir().unwrap();
    let (code, _, _) = run(&[
        "--input",
        FIXTURE,
        "--output",
        tmp.path().to_str().unwrap(),
        "--adapter",
        "notion",
    ]);
    assert_ne!(code, 0, "invalid adapter should fail");
}

#[test]
fn cli_fails_gracefully_on_missing_input() {
    let tmp = tempfile::tempdir().unwrap();
    let (code, _, _) = run(&[
        "--input",
        "/nonexistent/file.json",
        "--output",
        tmp.path().to_str().unwrap(),
    ]);
    assert_ne!(code, 0, "missing input file should fail");
}
