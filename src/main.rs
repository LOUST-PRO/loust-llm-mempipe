//! loust-llm-mempipe — CLI entrypoint.
//!
//! Full CLI surface is implemented in F4. This stub validates the binary
//! builds, the contract types resolve, and the version flag works.

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "loust-llm-mempipe",
    version,
    about = "Compile noisy LLM exports into token-efficient JSONL/Markdown",
    long_about = "Compile noisy LLM exports (ChatGPT, Claude, Gemini) into token-efficient \
                  JSONL + Markdown for Claude Code, Projects, and agent runtimes."
)]
struct Cli {
    /// Show extended build info and exit
    #[arg(long)]
    info: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.info {
        println!("loust-llm-mempipe {}", env!("CARGO_PKG_VERSION"));
        println!("repository: {}", env!("CARGO_PKG_REPOSITORY"));
        println!("rust-version: {}", env!("CARGO_PKG_RUST_VERSION"));
        println!("license: {}", env!("CARGO_PKG_LICENSE"));
        return Ok(());
    }

    // F4 will replace this with the real pipeline invocation.
    eprintln!(
        "loust-llm-mempipe {} — skeleton build (F1). Use --info for build details.",
        env!("CARGO_PKG_VERSION")
    );
    eprintln!("Full CLI arrives in F4. See the project plan for the roadmap.");

    Ok(())
}
