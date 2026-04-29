use ratatui::widgets::ListState;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::cli::*;
use crate::models::*;
use crate::tree::*;

pub(crate) struct TuiApp {
    pub(crate) db_path: PathBuf,
    pub(crate) export_base: PathBuf,
    pub(crate) limit: Option<usize>,
    pub(crate) search: String,
    pub(crate) search_mode: bool,
    pub(crate) status: String,
    pub(crate) last_export: Option<PathBuf>,
    pub(crate) index: OverviewIndex,
    pub(crate) expanded: HashSet<String>,
    pub(crate) visible_rows: Vec<VisibleRow>,
    pub(crate) list_state: ListState,
}

impl TuiApp {
    pub(crate) fn new(db_path: PathBuf, export_base: PathBuf, index: OverviewIndex, args: TuiArgs) -> Self {
        let mut app = Self {
            db_path,
            export_base,
            limit: args.limit,
            search: args.search.unwrap_or_default(),
            search_mode: false,
            status: String::from("ready · e export selected · E export root · o open last export"),
            last_export: None,
            index,
            expanded: HashSet::new(),
            visible_rows: Vec::new(),
            list_state: ListState::default(),
        };
        app.refresh_rows();
        app
    }

    pub(crate) fn selected_session_id(&self) -> Option<&str> {
        self.list_state
            .selected()
            .and_then(|index| self.visible_rows.get(index))
            .map(|row| row.session_id.as_str())
    }

    pub(crate) fn refresh_rows(&mut self) {
        self.visible_rows = build_visible_rows(&self.index, self.search.trim(), self.limit, &self.expanded);
        if self.visible_rows.is_empty() {
            self.list_state.select(None);
            return;
        }

        let current = self.list_state.selected().unwrap_or(0);
        self.list_state
            .select(Some(current.min(self.visible_rows.len().saturating_sub(1))));
    }

    pub(crate) fn move_selection(&mut self, delta: isize) {
        if self.visible_rows.is_empty() {
            self.list_state.select(None);
            return;
        }

        let current = self.list_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, self.visible_rows.len().saturating_sub(1) as isize) as usize;
        self.list_state.select(Some(next));
    }

    pub(crate) fn toggle_selected(&mut self) {
        if !self.search.trim().is_empty() {
            self.status = String::from("search mode auto-expands matching branches");
            return;
        }

        let Some(session_id) = self.selected_session_id().map(str::to_owned) else {
            return;
        };

        if self.index.children_of(&session_id).is_empty() {
            return;
        }

        if self.expanded.contains(&session_id) {
            self.expanded.remove(&session_id);
        } else {
            self.expanded.insert(session_id);
        }

        self.refresh_rows();
    }

    pub(crate) fn expand_all(&mut self) {
        self.expanded = self.index.all_expandable_ids();
        self.refresh_rows();
        self.status = String::from("expanded all");
    }

    pub(crate) fn collapse_all(&mut self) {
        self.expanded.clear();
        self.refresh_rows();
        self.status = String::from("collapsed all");
    }
}
