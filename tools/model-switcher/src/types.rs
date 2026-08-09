use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Variants offered by the switcher.
pub const VARIANTS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];
/// Provider prefix required for work-mode model assignments.
pub const WORK_PROVIDER: &str = "sewer-axonhub-work/";

/// Result of applying a profile to agent markdown files.
#[derive(Debug, Clone, Default)]
pub struct ApplyResult {
    /// absolute path → changed line count
    pub files: BTreeMap<String, usize>,
    /// tier name → changed line count
    pub tiers: BTreeMap<String, usize>,
    pub lines: usize,
}

/// Repo-relative paths discovered by walking upward from CWD.
#[derive(Debug, Clone)]
pub struct Env {
    pub root: String,
    pub tier_file: String,
    pub agent_dirs: Vec<String>,
}

/// Config paired with canonical tier order discovered at load time.
#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub tier_order: Vec<String>,
    pub profiles: Config,
}

/// Maps profile names to their tier→model assignments.
/// Mirrors the structure of model-switcher.json.
pub type Config = BTreeMap<String, TierSet>;

/// A single profile's tier→model map.
pub type TierSet = BTreeMap<String, Assignment>;

/// Model and reasoning variant assigned to one tier.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Assignment {
    pub model: String,
    pub variant: String,
}
