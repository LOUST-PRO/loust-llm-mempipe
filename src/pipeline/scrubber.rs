//! Rule E secret / PII scrubber. Walks the PipelineConfig.secret_patterns
//! list and replaces matches with `[REDACTED:<kind>]`. Implementation
//! in F3; constants live in `PipelineConfig::with_safe_defaults`.
