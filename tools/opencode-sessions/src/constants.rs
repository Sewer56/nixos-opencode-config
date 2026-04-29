

pub(crate) const TEXT_PREVIEW_LIMIT: usize = 160;

pub(crate) const INLINE_TEXT_PREVIEW_LIMIT: usize = 400;

pub(crate) const REASONING_PREVIEW_LIMIT: usize = 160;

pub(crate) const REASONING_SUMMARY_LIMIT: usize = 320;

pub(crate) const ACTIVITY_SUMMARY_LIMIT: usize = 220;

pub(crate) const ACTIVITY_SUMMARY_ITEMS_LIMIT: usize = 3;

pub(crate) const TURN_PREVIEW_LIMIT: usize = 220;

pub(crate) const TOOL_TEXT_PREVIEW_LIMIT: usize = 160;

pub(crate) const TOOL_INPUT_STRING_LIMIT: usize = 120;

pub(crate) const TOOL_INPUT_ITEMS_LIMIT: usize = 6;

pub(crate) const TOOL_INPUT_DEPTH_LIMIT: usize = 3;

pub(crate) const TOOL_INPUT_INLINE_CHARS_THRESHOLD: usize = 2_500;

pub(crate) const TOOL_PART_PREVIEW_LIMIT: usize = 140;

pub(crate) const SUBTASK_PREVIEW_LIMIT: usize = 200;

pub(crate) const PATCH_FILE_SAMPLE_LIMIT: usize = 6;

pub(crate) const SESSION_HOT_MESSAGES_LIMIT: usize = 5;

pub(crate) const EXPORT_HOTSPOT_LIMIT: usize = 3;

pub(crate) const TOOL_ROLLUP_DELTA_LIMIT: usize = 12;

pub(crate) const TURN_DELTA_LIMIT: usize = 12;

pub(crate) const TURN_FILE_SAMPLE_LIMIT: usize = 1;

pub(crate) const FILE_ACCESS_ROLLUP_LIMIT: usize = 16;

pub(crate) const ERROR_PATTERN_LIMIT: usize = 12;

pub(crate) const PATH_FALLBACK_COMPONENTS: usize = 6;

pub(crate) const MESSAGES_EMBEDDED_TEXT_LIMIT: usize = 2_000;

pub(crate) const ASSISTANT_TEXT_ARTIFACT_CHARS_THRESHOLD: usize = 400;

pub(crate) const REASONING_ARTIFACT_CHARS_THRESHOLD: usize = 6_000;

pub(crate) const TOOL_CALLS_EMBEDDED_IO_LIMIT: usize = 4_000;

pub(crate) const HOT_SESSION_TOKEN_THRESHOLD: u64 = 100_000;

pub(crate) const HOT_TURN_TOKEN_THRESHOLD: u64 = 500_000;

pub(crate) const HOT_MESSAGE_TOKEN_THRESHOLD: u64 = 50_000;

pub(crate) const HOT_TURN_TOOL_COUNT_THRESHOLD: usize = 3;

pub(crate) const HOT_MESSAGE_TOOL_COUNT_THRESHOLD: usize = 3;

pub(crate) const HOT_TURN_SLOW_MS_THRESHOLD: i64 = 60_000;

pub(crate) const HOT_MESSAGE_SLOW_MS_THRESHOLD: i64 = 30_000;

pub(crate) const HOT_TOOL_SLOW_MS_THRESHOLD: i64 = 1_000;

pub(crate) const HOT_TOOL_OUTPUT_CHARS_THRESHOLD: usize = 5_000;

pub(crate) const SCHEMA_VERSION: &str = "1.18";
