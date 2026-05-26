use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use std::io::{Stdout, stdout};
use std::time::{Duration, Instant};
use time::OffsetDateTime;

use crate::discovery::{
    ClaudeDiscovery, CodexDiscovery, CombinedDiscovery, EmptyDiscovery, SessionDiscovery,
};
use crate::model::OverviewSnapshot;
use crate::sort::{SortMode, sort_sessions};
use crate::tui::{build_overview_rows, render_overview_widget};
use ratatui::{Terminal, backend::CrosstermBackend};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefreshStatus {
    Idle,
    Refreshing,
    Ready,
    Partial,
    Failed,
}

#[derive(Debug, Clone)]
pub struct RefreshState {
    pub status: RefreshStatus,
    pub current_snapshot: OverviewSnapshot,
    pub last_refresh_started_at: Option<OffsetDateTime>,
    pub last_refresh_completed_at: Option<OffsetDateTime>,
}

impl RefreshState {
    pub fn new() -> Self {
        Self {
            status: RefreshStatus::Idle,
            current_snapshot: OverviewSnapshot::empty(),
            last_refresh_started_at: None,
            last_refresh_completed_at: None,
        }
    }

    pub fn start_refresh(&mut self, now: OffsetDateTime) {
        self.status = RefreshStatus::Refreshing;
        self.last_refresh_started_at = Some(now);
    }

    pub fn finish_refresh(&mut self, snapshot: OverviewSnapshot, now: OffsetDateTime) {
        self.status = if snapshot.errors.is_empty() {
            RefreshStatus::Ready
        } else {
            RefreshStatus::Partial
        };
        self.current_snapshot = snapshot;
        self.last_refresh_completed_at = Some(now);
    }
}

pub struct App {
    refresh_state: RefreshState,
    sort_mode: SortMode,
    diagnostic_mode: bool,
    refresh_interval: Duration,
}

impl App {
    pub fn new() -> Self {
        Self {
            refresh_state: RefreshState::new(),
            sort_mode: SortMode::Runtime,
            diagnostic_mode: false,
            refresh_interval: Duration::from_secs(5),
        }
    }

    pub fn set_sort_mode(&mut self, sort_mode: SortMode) {
        self.sort_mode = sort_mode;
    }

    pub fn sorted_sessions(&self) -> Vec<crate::model::Session> {
        sort_sessions(
            &self.refresh_state.current_snapshot.sessions,
            self.sort_mode,
        )
    }

    pub fn cycle_sort_mode(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Summary => SortMode::Runtime,
            SortMode::Runtime => SortMode::TokenUsage,
            SortMode::TokenUsage => SortMode::ContextLength,
            SortMode::ContextLength => SortMode::Summary,
        };
    }

    pub fn toggle_diagnostic_mode(&mut self) {
        self.diagnostic_mode = !self.diagnostic_mode;
    }

    pub fn refresh_with<D: SessionDiscovery>(&mut self, discovery: &D, now: OffsetDateTime) {
        self.refresh_state.start_refresh(now);

        match discovery.discover() {
            Ok(sessions) => {
                let snapshot = OverviewSnapshot {
                    captured_at: now,
                    sessions,
                    errors: vec![],
                };
                self.refresh_state.finish_refresh(snapshot, now);
            }
            Err(error) => {
                self.refresh_state.finish_refresh(
                    OverviewSnapshot {
                        captured_at: now,
                        sessions: self.refresh_state.current_snapshot.sessions.clone(),
                        errors: vec![error.message],
                    },
                    now,
                );
            }
        }
    }

    pub fn run(mut self) -> Result<()> {
        let mut providers: Vec<Box<dyn SessionDiscovery>> = Vec::new();
        if let Some(root) = ClaudeDiscovery::default_root() {
            providers.push(Box::new(ClaudeDiscovery::new(root)));
        }
        if let Some(root) = CodexDiscovery::default_root() {
            providers.push(Box::new(CodexDiscovery::new(root)));
        }

        if providers.is_empty() {
            let mut discovery = EmptyDiscovery;
            self.run_with_discovery(&mut discovery)
        } else {
            let mut discovery = CombinedDiscovery::new(providers);
            self.run_with_discovery(&mut discovery)
        }
    }

    pub fn run_with_discovery<D: SessionDiscovery>(&mut self, discovery: &mut D) -> Result<()> {
        let mut terminal = setup_terminal()?;
        let result = self.run_event_loop(&mut terminal, discovery);
        restore_terminal(&mut terminal)?;
        result
    }

    fn run_event_loop<D: SessionDiscovery>(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        discovery: &mut D,
    ) -> Result<()> {
        self.refresh_with(discovery, OffsetDateTime::now_utc());
        let mut last_refresh = Instant::now();

        loop {
            terminal.draw(|frame| {
                let size = frame.area();
                let rows = build_overview_rows(&self.sorted_sessions());
                let claude_count = self
                    .refresh_state
                    .current_snapshot
                    .provider_count("claude-code");
                let codex_count = self.refresh_state.current_snapshot.provider_count("codex");
                let refreshed = self
                    .refresh_state
                    .last_refresh_completed_at
                    .map(|ts| {
                        ts.format(&time::format_description::well_known::Rfc3339)
                            .unwrap_or_else(|_| "unknown".to_string())
                    })
                    .unwrap_or_else(|| "never".to_string());
                let widget = render_overview_widget(
                    &rows,
                    self.sort_mode,
                    claude_count,
                    codex_count,
                    &refreshed,
                    self.diagnostic_mode,
                );
                frame.render_widget(widget, size);
            })?;

            if last_refresh.elapsed() >= self.refresh_interval {
                self.refresh_with(discovery, OffsetDateTime::now_utc());
                last_refresh = Instant::now();
            }

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('s') => self.cycle_sort_mode(),
                        KeyCode::Char('d') => self.toggle_diagnostic_mode(),
                        KeyCode::Char('r') => {
                            self.refresh_with(discovery, OffsetDateTime::now_utc());
                            last_refresh = Instant::now();
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{App, RefreshState, RefreshStatus};
    use crate::discovery::{DiscoveryError, SessionDiscovery};
    use crate::model::{OverviewSnapshot, Session};
    use crate::sort::SortMode;
    use time::macros::datetime;

    struct StubDiscovery {
        sessions: Result<Vec<Session>, DiscoveryError>,
    }

    impl SessionDiscovery for StubDiscovery {
        fn discover(&self) -> Result<Vec<Session>, DiscoveryError> {
            self.sessions.clone()
        }
    }

    #[test]
    fn starts_in_idle_state_with_empty_snapshot() {
        let state = RefreshState::new();

        assert_eq!(state.status, RefreshStatus::Idle);
        assert!(state.current_snapshot.sessions.is_empty());
    }

    #[test]
    fn start_refresh_marks_state_refreshing() {
        let mut state = RefreshState::new();
        let now = datetime!(2026-05-25 10:00:00 UTC);

        state.start_refresh(now);

        assert_eq!(state.status, RefreshStatus::Refreshing);
        assert_eq!(state.last_refresh_started_at, Some(now));
    }

    #[test]
    fn finish_refresh_sets_ready_without_errors() {
        let mut state = RefreshState::new();
        let now = datetime!(2026-05-25 10:00:00 UTC);
        let snapshot = OverviewSnapshot {
            captured_at: now,
            sessions: vec![Session::minimal("session-1")],
            errors: vec![],
        };

        state.finish_refresh(snapshot.clone(), now);

        assert_eq!(state.status, RefreshStatus::Ready);
        assert_eq!(state.current_snapshot.sessions.len(), 1);
        assert_eq!(state.last_refresh_completed_at, Some(now));
    }

    #[test]
    fn app_returns_sessions_in_selected_sort_order() {
        let mut app = App::new();
        let now = datetime!(2026-05-25 10:00:00 UTC);

        let mut low = Session::minimal("low");
        low.token_usage = Some(100);

        let mut high = Session::minimal("high");
        high.token_usage = Some(900);

        app.refresh_state.finish_refresh(
            OverviewSnapshot {
                captured_at: now,
                sessions: vec![low, high],
                errors: vec![],
            },
            now,
        );
        app.set_sort_mode(SortMode::TokenUsage);

        let sorted = app.sorted_sessions();

        assert_eq!(sorted[0].session_id, "high");
        assert_eq!(sorted[1].session_id, "low");
    }

    #[test]
    fn refresh_replaces_snapshot_with_new_sessions() {
        let mut app = App::new();
        let now = datetime!(2026-05-25 10:05:00 UTC);
        let discovery = StubDiscovery {
            sessions: Ok(vec![Session::minimal("new-session")]),
        };

        app.refresh_with(&discovery, now);

        assert_eq!(app.refresh_state.status, RefreshStatus::Ready);
        assert_eq!(app.refresh_state.current_snapshot.sessions.len(), 1);
        assert_eq!(
            app.refresh_state.current_snapshot.sessions[0].session_id,
            "new-session"
        );
    }

    #[test]
    fn refresh_keeps_previous_sessions_and_marks_partial_on_error() {
        let mut app = App::new();
        let now = datetime!(2026-05-25 10:00:00 UTC);
        app.refresh_state.finish_refresh(
            OverviewSnapshot {
                captured_at: now,
                sessions: vec![Session::minimal("existing")],
                errors: vec![],
            },
            now,
        );

        let later = datetime!(2026-05-25 10:06:00 UTC);
        let discovery = StubDiscovery {
            sessions: Err(DiscoveryError {
                message: "metadata unavailable".to_string(),
            }),
        };

        app.refresh_with(&discovery, later);

        assert_eq!(app.refresh_state.status, RefreshStatus::Partial);
        assert_eq!(app.refresh_state.current_snapshot.sessions.len(), 1);
        assert_eq!(
            app.refresh_state.current_snapshot.sessions[0].session_id,
            "existing"
        );
        assert_eq!(
            app.refresh_state.current_snapshot.errors,
            vec!["metadata unavailable"]
        );
    }

    #[test]
    fn cycle_sort_mode_rotates_through_supported_modes() {
        let mut app = App::new();

        assert_eq!(app.sort_mode, SortMode::Runtime);
        app.cycle_sort_mode();
        assert_eq!(app.sort_mode, SortMode::TokenUsage);
        app.cycle_sort_mode();
        assert_eq!(app.sort_mode, SortMode::ContextLength);
        app.cycle_sort_mode();
        assert_eq!(app.sort_mode, SortMode::Summary);
    }

    #[test]
    fn toggle_diagnostic_mode_switches_state() {
        let mut app = App::new();

        assert!(!app.diagnostic_mode);
        app.toggle_diagnostic_mode();
        assert!(app.diagnostic_mode);
        app.toggle_diagnostic_mode();
        assert!(!app.diagnostic_mode);
    }
}
