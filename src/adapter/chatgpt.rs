//! ChatGPT export adapter. Implementation lands in F2.
//!
//! Expected input format: ChatGPT data export (Settings → Data Controls → Export Data).
//! Top-level shape: `{ "conversations": [ { id, title, create_time, mapping: { ... } } ] }`
//! The `mapping` is a non-linear node tree; the adapter must reconstruct linear threads.

use crate::adapter::{Adapter, AdapterKind};
use crate::pipeline::NormalizedMessage;
use std::io::Read;

pub struct ChatGptAdapter;

impl Adapter for ChatGptAdapter {
    fn kind(&self) -> AdapterKind {
        AdapterKind::ChatGpt
    }
    fn detect(&self, _header: &[u8]) -> bool {
        false /* F2: sniff for "conversations" */
    }
    fn stream_messages(
        &self,
        _reader: Box<dyn Read>,
    ) -> anyhow::Result<Box<dyn Iterator<Item = NormalizedMessage>>> {
        anyhow::bail!("ChatGptAdapter not yet implemented (F2)")
    }
}
