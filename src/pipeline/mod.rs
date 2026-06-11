//! Pipeline orchestrator. The pipeline takes a stream of
//! `NormalizedMessage`s from any `Adapter` and produces token-efficient
//! output (JSONL or Markdown) ready for Claude Code / Projects.

use chrono::{DateTime, Utc};
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::hash::Hasher;

pub mod dedup;
pub mod normalizer;
pub mod parser;
pub mod scrubber;
pub mod signals;
pub mod writer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
            Self::Tool => "tool",
        }
    }
}

/// The canonical message shape that every Adapter must produce.
/// Downstream stages (scrubber, dedup, signals, writer) only know
/// this type — adapters are decoupled from each other.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedMessage {
    /// Stable identifier within the source (e.g. ChatGPT node UUID,
    /// Claude message UUID, JSONL line id).
    pub id: String,

    /// Source system identifier. Mirrors `AdapterKind::as_str()` in
    /// the serialized form for downstream grep-ability.
    pub source: String,

    /// Message role. Tool messages are kept for completeness but
    /// weighted low in `signals::signal_score`.
    pub role: Role,

    /// Scrubbed message body. Rule E applied upstream.
    pub content: String,

    /// Original (unscrubbed) length, kept for stats reporting.
    pub original_length: u32,

    /// When the message was created in the source system.
    pub created_at: DateTime<Utc>,

    /// Source-specific thread identifier. Used to group messages
    /// in Markdown output.
    pub thread_id: String,

    /// Optional human-meaningful thread title (e.g. ChatGPT's
    /// `conversation.title`). Used for the Markdown file naming.
    pub thread_title: Option<String>,

    /// Heuristic project assignment (e.g. detected from first message
    /// or user-assigned tags). Used to group Markdown files.
    pub project_hint: Option<String>,

    /// FNV-1a 64-bit hash of the scrubbed content. Used as the
    /// first-pass dedup key before Jaccard similarity kicks in.
    pub content_hash: u64,
}

impl NormalizedMessage {
    /// Compute the FNV-1a-like content hash via `seahash`. Stable
    /// across runs and architectures (no random seed).
    pub fn compute_content_hash(content: &str) -> u64 {
        let mut hasher = SeaHasher::new();
        hasher.write(content.as_bytes());
        hasher.finish()
    }

    /// Slugify a string for use in a filename. Conservative:
    /// lowercase, alphanumeric + dash, max 64 chars.
    pub fn slugify(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut last_dash = false;
        for ch in input.chars() {
            let ch_lower = ch.to_ascii_lowercase();
            if ch_lower.is_ascii_alphanumeric() {
                out.push(ch_lower);
                last_dash = false;
            } else if !last_dash && !out.is_empty() {
                out.push('-');
                last_dash = true;
            }
            if out.len() >= 64 {
                break;
            }
        }
        let trimmed = out.trim_end_matches('-').to_string();
        if trimmed.is_empty() {
            "untitled".to_string()
        } else {
            trimmed
        }
    }
}
