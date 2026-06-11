// Sorting helpers currently unused — kept for potential TUI expansion.
// Remove #[allow(dead_code)] blocks when callers are added.

#[allow(dead_code)]
use crate::types::CountItem;

#[allow(dead_code)]
pub fn sorted_counts(counts: &std::collections::BTreeMap<String, usize>) -> Vec<CountItem> {
    let mut items: Vec<CountItem> = counts
        .iter()
        .map(|(model, &count)| CountItem {
            model: model.clone(),
            count,
        })
        .collect();
    items.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.model.cmp(&b.model)));
    items
}

#[allow(dead_code)]
pub fn sorted_file_keys(files: &std::collections::BTreeMap<String, usize>) -> Vec<&String> {
    files.keys().collect()
}
