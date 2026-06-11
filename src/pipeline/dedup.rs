//! Two-stage dedup. Stage 1: exact match via FNV-1a `content_hash`.
//! Stage 2: Jaccard token similarity (threshold from `PipelineConfig`).
//! Implementation lands in F3.
