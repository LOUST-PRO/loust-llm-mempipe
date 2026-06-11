//! Claude.ai web takeout adapter. Implementation lands post-MVP.

use crate::adapter::{Adapter, AdapterKind};
use crate::pipeline::NormalizedMessage;
use std::io::Read;

pub struct ClaudeWebAdapter;

impl Adapter for ClaudeWebAdapter {
    fn kind(&self) -> AdapterKind {
        AdapterKind::ClaudeWeb
    }
    fn detect(&self, _header: &[u8]) -> bool {
        false
    }
    fn stream_messages(
        &self,
        _reader: Box<dyn Read>,
    ) -> anyhow::Result<Box<dyn Iterator<Item = NormalizedMessage>>> {
        anyhow::bail!("ClaudeWebAdapter not yet implemented")
    }
}
