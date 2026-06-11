//! Claude Code JSONL local adapter. Reads the line-delimited format
//! already produced by `~/.claude/projects/*/sessions/*.jsonl` and
//! normalizes to the same shape. Loop virtuous with the internal
//! `lzt-chats` FTS5 indexer.

use crate::adapter::{Adapter, AdapterKind};
use crate::pipeline::NormalizedMessage;
use std::io::Read;

pub struct ClaudeCodeAdapter;

impl Adapter for ClaudeCodeAdapter {
    fn kind(&self) -> AdapterKind {
        AdapterKind::ClaudeCode
    }
    fn detect(&self, _header: &[u8]) -> bool {
        false
    }
    fn stream_messages(
        &self,
        _reader: Box<dyn Read>,
    ) -> anyhow::Result<Box<dyn Iterator<Item = NormalizedMessage>>> {
        anyhow::bail!("ClaudeCodeAdapter not yet implemented")
    }
}
