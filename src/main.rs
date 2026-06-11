//! loust-llm-mempipe — CLI entrypoint.
//!
//! F4 surface:
//!
//! ```text
//! loust-llm-mempipe \
//!     --input conversations.json \
//!     --output ./claude-memory/ \
//!     --format both \
//!     --stats
//! ```
//!
//! Flags:
//! - `--input` (required) — path to the raw export file
//! - `--output` (required) — directory to write outputs to
//! - `--format` — `jsonl` (default), `markdown`/`md`, or `both`
//! - `--adapter` — force a specific adapter (default: auto-detect)
//! - `--dedup-threshold` — Jaccard threshold (default 0.85)
//! - `--signal-min` — drop survivors with `signal_score < min` (default 0.2)
//! - `--max-age-days` — drop messages older than N days (default 1095 = 3y)
//! - `--stats` — print one-line stats to stderr
//! - `--dry-run` — compute but don't write
//! - `--info` — print build info and exit
//! - `--version`, `--help` — clap defaults

use anyhow::{Context, Result};
use clap::Parser;
use loust_llm_mempipe::adapter::{pick_adapter, AdapterKind};
use loust_llm_mempipe::pipeline::{parser, writer};
use loust_llm_mempipe::{OutputFormat, Pipeline, PipelineConfig};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "loust-llm-mempipe",
    version,
    about = "Compile noisy LLM exports into token-efficient JSONL/Markdown",
    long_about = "Compile noisy LLM exports (ChatGPT, Claude, Gemini) into token-efficient \
                  JSONL + Markdown for Claude Code, Projects, and agent runtimes."
)]
struct Cli {
    /// Path to the raw export file (e.g. ChatGPT `conversations.json`).
    #[arg(short, long)]
    input: PathBuf,

    /// Directory to write outputs to. Created if missing.
    #[arg(short, long)]
    output: PathBuf,

    /// Output format. `jsonl` is one line per `NormalizedMessage`
    /// (consumable by `claude-code --context`). `markdown` writes a
    /// hierarchical `<dir>/<project>/<thread>.md` tree. `both` writes
    /// both.
    #[arg(short, long, default_value = "jsonl", value_parser = parse_format)]
    format: OutputFormat,

    /// Force a specific adapter instead of auto-detecting. Accepted
    /// values: `chatgpt`, `claude_web`, `gemini`, `claude_code`.
    #[arg(long, value_parser = parse_adapter_kind)]
    adapter: Option<AdapterKind>,

    /// Jaccard token-similarity threshold for fuzzy dedup. Two messages
    /// with similarity > threshold are collapsed. Range [0.0, 1.0].
    #[arg(long, default_value_t = 0.85)]
    dedup_threshold: f64,

    /// Drop survivors with `signal_score < min`. Lower = keep more
    /// (and noisier) context. Range [0.0, 1.0].
    #[arg(long, default_value_t = 0.2)]
    signal_min: f64,

    /// Drop messages older than N days. 1095 = ~3 years.
    #[arg(long, default_value_t = 1095)]
    max_age_days: u64,

    /// Print one-line stats to stderr after the run.
    #[arg(long)]
    stats: bool,

    /// Compute the pipeline but don't write any files. Useful with
    /// `--stats` to dry-validate a config against an export.
    #[arg(long)]
    dry_run: bool,

    /// Show extended build info and exit.
    #[arg(long)]
    info: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.info {
        print_info();
        return Ok(());
    }

    // Build the config from CLI args, defaulting to safe presets and
    // overriding only what the user asked to change.
    let mut config = PipelineConfig::with_safe_defaults();
    config.dedup_threshold = cli.dedup_threshold;
    config.signal_min = cli.signal_min;
    config.max_thread_age_days = cli.max_age_days;
    config.output_format = cli.format;
    config.dry_run = cli.dry_run;

    // Read the whole input. For F4 MVP we accept the in-memory cost;
    // 50 MB exports -> ~100 MB peak, fine on a developer laptop. The
    // F2 ChatGPT adapter already materializes the full JSON tree.
    let input_path = &cli.input;
    let bytes = std::fs::read(input_path)
        .with_context(|| format!("reading input file {}", input_path.display()))?;
    let header: &[u8] = if bytes.len() <= 4096 {
        &bytes
    } else {
        &bytes[..4096]
    };

    // Pick the adapter: explicit override, or first registry match.
    let adapter = pick_adapter(cli.adapter, header).with_context(|| {
        format!(
            "no adapter can parse the input file (size={} bytes). \
             Try `--adapter chatgpt` to force one. Available adapters: {}",
            bytes.len(),
            available_adapters_list(),
        )
    })?;
    let adapter_label = adapter.kind().as_str();

    eprintln!("detected adapter: {}", adapter_label);

    // Parse → pipeline → output.
    let source_label = input_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(adapter_label);
    // Own the bytes inside a Cursor so the `Box<dyn Read>` doesn't
    // need a `'static` borrow. The Cursor drains in place; we move
    // `bytes` into it. After this line, `bytes` is no longer accessible
    // (good — it would be a use-after-free otherwise).
    let cursor = std::io::Cursor::new(bytes);
    let messages = parser::parse(adapter.as_ref(), Box::new(cursor), source_label)?;
    eprintln!("parsed {} messages", messages.len());

    // Save the format before `Pipeline::new` consumes `config`.
    let out_format = config.output_format;
    let output = Pipeline::new(config).run(messages, chrono::Utc::now());

    if cli.stats {
        eprintln!("stats: {}", output.stats.one_line());
    }

    if cli.dry_run {
        eprintln!("dry run: nothing written");
        return Ok(());
    }

    std::fs::create_dir_all(&cli.output)
        .with_context(|| format!("creating output dir {}", cli.output.display()))?;
    let written = writer::write_all(&cli.output, &output, out_format)
        .with_context(|| format!("writing to {}", cli.output.display()))?;
    for path in &written {
        eprintln!("wrote: {}", path.display());
    }
    eprintln!("done: {} files written", written.len());

    Ok(())
}

fn print_info() {
    println!("loust-llm-mempipe {}", env!("CARGO_PKG_VERSION"));
    println!("repository: {}", env!("CARGO_PKG_REPOSITORY"));
    println!("rust-version: {}", env!("CARGO_PKG_RUST_VERSION"));
    println!("license: {}", env!("CARGO_PKG_LICENSE"));
    println!("description: {}", env!("CARGO_PKG_DESCRIPTION"));
}

fn available_adapters_list() -> String {
    use loust_llm_mempipe::adapter::registry;
    registry()
        .iter()
        .map(|a| a.kind().as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_format(s: &str) -> Result<OutputFormat, String> {
    OutputFormat::from_cli(s).ok_or_else(|| {
        format!(
            "invalid format '{}': expected one of jsonl, markdown, both",
            s
        )
    })
}

fn parse_adapter_kind(s: &str) -> Result<AdapterKind, String> {
    AdapterKind::from_cli(s).ok_or_else(|| {
        format!(
            "invalid adapter '{}': expected one of chatgpt, claude_web, gemini, claude_code",
            s
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn clap_parses_minimal_required_args() {
        let cli = Cli::try_parse_from([
            "loust-llm-mempipe",
            "--input",
            "in.json",
            "--output",
            "out/",
        ])
        .unwrap();
        assert_eq!(cli.input, PathBuf::from("in.json"));
        assert_eq!(cli.output, PathBuf::from("out/"));
        assert_eq!(cli.format, OutputFormat::Jsonl);
        assert!(cli.adapter.is_none());
        assert!((cli.dedup_threshold - 0.85).abs() < f64::EPSILON);
        assert!((cli.signal_min - 0.2).abs() < f64::EPSILON);
        assert_eq!(cli.max_age_days, 1095);
        assert!(!cli.stats);
        assert!(!cli.dry_run);
    }

    #[test]
    fn clap_parses_all_overrides() {
        let cli = Cli::try_parse_from([
            "loust-llm-mempipe",
            "--input",
            "in.json",
            "--output",
            "out/",
            "--format",
            "both",
            "--adapter",
            "chatgpt",
            "--dedup-threshold",
            "0.9",
            "--signal-min",
            "0.3",
            "--max-age-days",
            "365",
            "--stats",
            "--dry-run",
        ])
        .unwrap();
        assert_eq!(cli.format, OutputFormat::Both);
        assert_eq!(cli.adapter, Some(AdapterKind::ChatGpt));
        assert!((cli.dedup_threshold - 0.9).abs() < f64::EPSILON);
        assert!((cli.signal_min - 0.3).abs() < f64::EPSILON);
        assert_eq!(cli.max_age_days, 365);
        assert!(cli.stats);
        assert!(cli.dry_run);
    }

    #[test]
    fn clap_rejects_invalid_format() {
        let result = Cli::try_parse_from([
            "loust-llm-mempipe",
            "--input",
            "in.json",
            "--output",
            "out/",
            "--format",
            "xml",
        ]);
        assert!(result.is_err(), "should reject unknown format");
    }

    #[test]
    fn clap_rejects_invalid_adapter() {
        let result = Cli::try_parse_from([
            "loust-llm-mempipe",
            "--input",
            "in.json",
            "--output",
            "out/",
            "--adapter",
            "notion",
        ]);
        assert!(result.is_err(), "should reject unknown adapter");
    }
}
