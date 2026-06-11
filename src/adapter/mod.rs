//! Source adapters. Each adapter knows how to parse one specific export
//! format (ChatGPT JSON, Claude Web takeout, Gemini takeout, Claude Code
//! JSONL) into the common `NormalizedMessage` stream.

use crate::pipeline::NormalizedMessage;
use std::io::Read;

pub mod chatgpt;
pub mod claude_code;
pub mod claude_web;
pub mod gemini;

/// Identifier for the source system of a message. Used in the output
/// schema so downstream consumers (RAG indexers, Claude Projects) can
/// filter or weight by source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdapterKind {
    ChatGpt,
    ClaudeWeb,
    Gemini,
    ClaudeCode,
}

impl AdapterKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ChatGpt => "chatgpt",
            Self::ClaudeWeb => "claude_web",
            Self::Gemini => "gemini",
            Self::ClaudeCode => "claude_code",
        }
    }
}

/// A source-specific parser that streams `NormalizedMessage`s out of a
/// raw export file. Implementations MUST be streaming (no full-file
/// `serde_json::from_reader`) so 50MB+ exports don't OOM.
///
/// The `detect` method inspects the first few KB of the file and
/// returns true if this adapter can parse it. The CLI in F4 will
/// auto-select based on detection, with explicit override available.
pub trait Adapter {
    fn kind(&self) -> AdapterKind;

    /// Returns true if this adapter recognizes the file header.
    /// `header` is the first ~4KB of the file.
    fn detect(&self, header: &[u8]) -> bool;

    /// Streams NormalizedMessages from the reader. Implementations
    /// are responsible for thread reconstruction, role normalization,
    /// and skipping empty / system / error messages.
    fn stream_messages(
        &self,
        reader: Box<dyn Read>,
    ) -> anyhow::Result<Box<dyn Iterator<Item = NormalizedMessage>>>;
}
