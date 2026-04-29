use anyhow::{Context, Result};
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::fs::{self};
use std::io::{self};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::cli::*;
use crate::models::*;
use crate::db::*;
use crate::export::*;
use crate::tui::app::*;
use crate::tui::render::*;

pub(crate) fn run_tui(db_path: PathBuf, index: OverviewIndex, args: TuiArgs) -> Result<()> {
    let export_base = default_export_base_dir();
    fs::create_dir_all(&export_base).with_context(|| format!("create {}", export_base.display()))?;

    enable_raw_mode().context("enable raw mode")?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen).context("enter alt screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("create terminal")?;
    let mut app = TuiApp::new(db_path.clone(), export_base, index, args);

    let result = (|| -> Result<()> {
        loop {
            terminal.draw(|frame| draw_tui(frame, &mut app)).context("draw tui")?;

            if !event::poll(Duration::from_millis(250)).context("poll terminal events")? {
                continue;
            }

            let Event::Key(key) = event::read().context("read terminal event")? else {
                continue;
            };

            if key.kind != KeyEventKind::Press {
                continue;
            }

            if handle_tui_key(&mut app, key)? {
                break;
            }
        }

        Ok(())
    })();

    disable_raw_mode().ok();
    let mut stdout = io::stdout();
    stdout.execute(LeaveAlternateScreen).ok();
    result
}

pub(crate) fn handle_tui_key(app: &mut TuiApp, key: KeyEvent) -> Result<bool> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Ok(true);
    }

    if app.search_mode {
        match key.code {
            KeyCode::Esc => {
                app.search_mode = false;
                app.status = String::from("search mode off");
            }
            KeyCode::Enter => {
                app.search_mode = false;
                app.status = format!("search = {}", app.search.trim());
            }
            KeyCode::Backspace => {
                app.search.pop();
                app.refresh_rows();
            }
            KeyCode::Char(ch) => {
                app.search.push(ch);
                app.refresh_rows();
            }
            _ => {}
        }
        return Ok(false);
    }

    match key.code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
        KeyCode::Enter | KeyCode::Char(' ') => app.toggle_selected(),
        KeyCode::Char('/') => {
            app.search_mode = true;
            app.status = String::from("search mode on");
        }
        KeyCode::Char('a') => app.expand_all(),
        KeyCode::Char('z') => app.collapse_all(),
        KeyCode::Esc
            if !app.search.is_empty() => {
                app.search.clear();
                app.refresh_rows();
                app.status = String::from("search cleared");
            }
        KeyCode::Char('e') => {
            let Some(session_id) = app.selected_session_id().map(str::to_owned) else {
                app.status = String::from("nothing selected");
                return Ok(false);
            };
            let conn = open_db(&app.db_path)?;
            let export_root = export_bundle(&conn, &app.index, &session_id, Some(app.export_base.clone()))?;
            app.last_export = Some(export_root.clone());
            app.status = format!("exported selected -> {}", export_root.display());
        }
        KeyCode::Char('E') => {
            let Some(session_id) = app.selected_session_id().map(str::to_owned) else {
                app.status = String::from("nothing selected");
                return Ok(false);
            };
            let root_id = app.index.root_id(&session_id)?;
            let conn = open_db(&app.db_path)?;
            let export_root = export_bundle(&conn, &app.index, &root_id, Some(app.export_base.clone()))?;
            app.last_export = Some(export_root.clone());
            app.status = format!("exported root -> {}", export_root.display());
        }
        KeyCode::Char('o') => {
            let Some(path) = app.last_export.as_ref() else {
                app.status = String::from("no export yet");
                return Ok(false);
            };
            open_path(path)?;
            app.status = format!("opened -> {}", path.display());
        }
        _ => {}
    }

    Ok(false)
}

pub(crate) fn open_path(path: &Path) -> Result<()> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };

    std::process::Command::new(opener)
        .arg(path)
        .spawn()
        .with_context(|| format!("launch {opener} for {}", path.display()))?;
    Ok(())
}
