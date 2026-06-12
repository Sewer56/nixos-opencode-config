use std::collections::BTreeMap;

/// Provider prefix required for work-mode model assignments.
pub const WORK_PROVIDER: &str = "sewer-axonhub-work/";

/// Maps profile names to their tier→model assignments.
/// Mirrors the structure of model-tiers.json.
pub type Config = BTreeMap<String, TierSet>;

/// A single profile's tier→model map.
pub type TierSet = BTreeMap<String, String>;

/// Config paired with canonical tier order discovered at load time.
#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub tier_order: Vec<String>,
    pub profiles: Config,
}

/// Repo-relative paths discovered by walking upward from CWD.
#[derive(Debug, Clone)]
pub struct Env {
    pub root: String,
    pub tier_file: String,
    pub agent_dirs: Vec<String>,
}

/// Result of applying a profile to agent markdown files.
#[derive(Debug, Clone, Default)]
pub struct ApplyResult {
    /// absolute path → changed line count
    pub files: BTreeMap<String, usize>,
    /// tier name → changed line count
    pub tiers: BTreeMap<String, usize>,
    pub lines: usize,
}

