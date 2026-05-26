use crate::model::Session;
use crate::sort::SortMode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Widget},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRow {
    pub provider: String,
    pub source: String,
    pub activity_state: String,
    pub title: String,
    pub summary: String,
    pub runtime: String,
    pub token_usage: String,
    pub context_length: String,
    pub message_count: String,
    pub cwd: String,
    pub match_reason: String,
    pub dedupe_key: String,
    pub hidden_duplicates: String,
}

pub struct OverviewWidget<'a> {
    rows: &'a [SessionRow],
    sort_mode: SortMode,
    claude_count: usize,
    codex_count: usize,
    last_refresh_label: &'a str,
    diagnostic_mode: bool,
}

impl SessionRow {
    pub fn from_session(session: &Session) -> Self {
        Self {
            provider: session
                .provider
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            source: session
                .source
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            activity_state: session
                .activity_state
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            title: session
                .title
                .clone()
                .unwrap_or_else(|| "Unavailable".to_string()),
            summary: session
                .summary
                .clone()
                .unwrap_or_else(|| "Unavailable".to_string()),
            runtime: session
                .runtime_seconds
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Unavailable".to_string()),
            token_usage: session
                .token_usage
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Unavailable".to_string()),
            context_length: session
                .context_length
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Unavailable".to_string()),
            message_count: session
                .message_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Unavailable".to_string()),
            cwd: session
                .cwd
                .clone()
                .unwrap_or_else(|| "Unavailable".to_string()),
            match_reason: session
                .match_reason
                .clone()
                .unwrap_or_else(|| "Unavailable".to_string()),
            dedupe_key: session
                .dedupe_key
                .clone()
                .unwrap_or_else(|| "Unavailable".to_string()),
            hidden_duplicates: session.hidden_duplicates.to_string(),
        }
    }
}

pub fn build_overview_rows(sessions: &[Session]) -> Vec<SessionRow> {
    sessions.iter().map(SessionRow::from_session).collect()
}

pub fn render_overview_text(rows: &[SessionRow]) -> String {
    rows.iter()
        .map(|row| {
            format!(
                "{} | {} | {} | {} | {} | {} | {}",
                row.provider,
                row.source,
                row.activity_state,
                row.title,
                row.summary,
                row.runtime,
                row.token_usage
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn build_status_line(
    rows: &[SessionRow],
    sort_mode: SortMode,
    claude_count: usize,
    codex_count: usize,
    last_refresh_label: &str,
    diagnostic_mode: bool,
) -> String {
    let hidden_duplicates: u64 = rows
        .iter()
        .filter_map(|row| row.hidden_duplicates.parse::<u64>().ok())
        .sum();
    format!(
        "active:{}  claude:{}  codex:{}  sort:{}  diagnostics:{}  hidden_dupes:{}  refreshed:{}",
        rows.len(),
        claude_count,
        codex_count,
        sort_mode.label(),
        if diagnostic_mode { "on" } else { "off" },
        hidden_duplicates,
        last_refresh_label
    )
}

pub fn build_help_line(diagnostic_mode: bool) -> String {
    format!(
        "Keys: [s] sort  [r] refresh  [d] diagnostics:{}  [q] quit",
        if diagnostic_mode { "on" } else { "off" }
    )
}

pub fn render_overview_widget<'a>(
    rows: &'a [SessionRow],
    sort_mode: SortMode,
    claude_count: usize,
    codex_count: usize,
    last_refresh_label: &'a str,
    diagnostic_mode: bool,
) -> OverviewWidget<'a> {
    OverviewWidget {
        rows,
        sort_mode,
        claude_count,
        codex_count,
        last_refresh_label,
        diagnostic_mode,
    }
}

impl Widget for OverviewWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = if self.diagnostic_mode {
            format!(
                "Local Session Overview [{}] [diagnostics]",
                self.sort_mode.label()
            )
        } else {
            format!("Local Session Overview [{}]", self.sort_mode.label())
        };
        let header_cells = if self.diagnostic_mode {
            vec![
                "Provider", "Source", "State", "Title", "Summary", "Runtime", "Tokens", "CWD",
                "Match", "Key", "Hidden",
            ]
        } else {
            vec![
                "Provider", "Source", "State", "Title", "Summary", "Runtime", "Tokens", "Context",
                "Msgs",
            ]
        };
        let header = Row::new(header_cells).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
        let body = self.rows.iter().map(|row| {
            let cells = if self.diagnostic_mode {
                vec![
                    Cell::from(row.provider.clone()),
                    Cell::from(row.source.clone()),
                    Cell::from(row.activity_state.clone()),
                    Cell::from(row.title.clone()),
                    Cell::from(row.summary.clone()),
                    Cell::from(row.runtime.clone()),
                    Cell::from(row.token_usage.clone()),
                    Cell::from(row.cwd.clone()),
                    Cell::from(row.match_reason.clone()),
                    Cell::from(row.dedupe_key.clone()),
                    Cell::from(row.hidden_duplicates.clone()),
                ]
            } else {
                vec![
                    Cell::from(row.provider.clone()),
                    Cell::from(row.source.clone()),
                    Cell::from(row.activity_state.clone()),
                    Cell::from(row.title.clone()),
                    Cell::from(row.summary.clone()),
                    Cell::from(row.runtime.clone()),
                    Cell::from(row.token_usage.clone()),
                    Cell::from(row.context_length.clone()),
                    Cell::from(row.message_count.clone()),
                ]
            };
            Row::new(cells).style(Style::default().fg(Color::White))
        });

        let widths: Vec<Constraint> = if self.diagnostic_mode {
            vec![
                Constraint::Percentage(8),
                Constraint::Percentage(10),
                Constraint::Percentage(7),
                Constraint::Percentage(10),
                Constraint::Percentage(14),
                Constraint::Percentage(6),
                Constraint::Percentage(7),
                Constraint::Percentage(16),
                Constraint::Percentage(8),
                Constraint::Percentage(10),
                Constraint::Percentage(4),
            ]
        } else {
            vec![
                Constraint::Percentage(9),
                Constraint::Percentage(12),
                Constraint::Percentage(8),
                Constraint::Percentage(14),
                Constraint::Percentage(20),
                Constraint::Percentage(8),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(9),
            ]
        };

        let table = Table::new(body, widths).header(header).block(
            Block::default()
                .title(title)
                .title_style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(Color::Blue))
                .borders(Borders::ALL),
        );

        let help = Paragraph::new(build_help_line(self.diagnostic_mode))
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );

        let status = Paragraph::new(build_status_line(
            self.rows,
            self.sort_mode,
            self.claude_count,
            self.codex_count,
            self.last_refresh_label,
            self.diagnostic_mode,
        ))
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(2),
                Constraint::Length(2),
            ])
            .split(area);
        table.render(chunks[0], buf);
        help.render(chunks[1], buf);
        status.render(chunks[2], buf);
    }
}

pub fn render_overview_buffer(rows: &[SessionRow], sort_mode: SortMode, area: Rect) -> Buffer {
    let mut buffer = Buffer::empty(area);
    render_overview_widget(rows, sort_mode, 0, 0, "never", false).render(area, &mut buffer);
    buffer
}

pub fn render_diagnostic_overview_buffer(
    rows: &[SessionRow],
    sort_mode: SortMode,
    area: Rect,
) -> Buffer {
    let mut buffer = Buffer::empty(area);
    render_overview_widget(rows, sort_mode, 0, 0, "never", true).render(area, &mut buffer);
    buffer
}

#[cfg(test)]
mod tests {
    use super::{
        SessionRow, build_help_line, build_overview_rows, build_status_line,
        render_diagnostic_overview_buffer, render_overview_buffer, render_overview_text,
    };
    use crate::model::Session;
    use crate::sort::SortMode;
    use ratatui::layout::Rect;

    #[test]
    fn missing_fields_render_as_unavailable() {
        let row = SessionRow::from_session(&Session::minimal("session-1"));

        assert_eq!(row.title, "Unavailable");
        assert_eq!(row.summary, "Unavailable");
        assert_eq!(row.runtime, "Unavailable");
    }

    #[test]
    fn overview_rows_preserve_available_fields() {
        let mut session = Session::minimal("session-1");
        session.title = Some("Agent A".to_string());
        session.summary = Some("Investigating session data".to_string());
        session.runtime_seconds = Some(42);
        session.token_usage = Some(500);
        session.context_length = Some(4096);
        session.message_count = Some(9);

        let rows = build_overview_rows(&[session]);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].provider, "unknown");
        assert_eq!(rows[0].source, "unknown");
        assert_eq!(rows[0].activity_state, "unknown");
        assert_eq!(rows[0].title, "Agent A");
        assert_eq!(rows[0].summary, "Investigating session data");
        assert_eq!(rows[0].token_usage, "500");
    }

    #[test]
    fn rendered_overview_contains_multiple_rows() {
        let mut first = Session::minimal("session-1");
        first.title = Some("First".to_string());
        first.summary = Some("One".to_string());

        let mut second = Session::minimal("session-2");
        second.title = Some("Second".to_string());
        second.summary = Some("Two".to_string());

        let rendered = render_overview_text(&build_overview_rows(&[first, second]));

        assert!(rendered.contains("unknown | unknown | unknown | First | One"));
        assert!(rendered.contains("unknown | unknown | unknown | Second | Two"));
    }

    #[test]
    fn status_line_contains_counts_and_refresh_info() {
        let mut session = Session::minimal("session-1");
        session.provider = Some("claude-code".to_string());
        let rows = build_overview_rows(&[session]);

        let status = build_status_line(&rows, SortMode::Runtime, 1, 0, "just now", false);

        assert!(status.contains("active:1"));
        assert!(status.contains("claude:1"));
        assert!(status.contains("sort:runtime"));
        assert!(status.contains("diagnostics:off"));
        assert!(status.contains("refreshed:just now"));
    }

    #[test]
    fn help_line_contains_key_hints() {
        let help = build_help_line(true);

        assert!(help.contains("[s] sort"));
        assert!(help.contains("[r] refresh"));
        assert!(help.contains("[d] diagnostics:on"));
        assert!(help.contains("[q] quit"));
    }

    #[test]
    fn diagnostic_rows_preserve_matching_details() {
        let mut session = Session::minimal("session-1");
        session.cwd = Some("/Users/rj/Code/aisess".to_string());
        session.match_reason = Some("process-cwd".to_string());
        session.dedupe_key = Some("/users/rj/code/aisess".to_string());
        session.hidden_duplicates = 2;

        let rows = build_overview_rows(&[session]);
        let status = build_status_line(&rows, SortMode::Runtime, 0, 0, "just now", true);

        assert_eq!(rows[0].cwd, "/Users/rj/Code/aisess");
        assert_eq!(rows[0].match_reason, "process-cwd");
        assert_eq!(rows[0].dedupe_key, "/users/rj/code/aisess");
        assert_eq!(rows[0].hidden_duplicates, "2");
        assert!(status.contains("diagnostics:on"));
        assert!(status.contains("hidden_dupes:2"));
    }

    #[test]
    fn overview_buffer_contains_screen_title_and_session_content() {
        let mut session = Session::minimal("session-1");
        session.provider = Some("codex".to_string());
        session.source = Some("codex-cli".to_string());
        session.activity_state = Some("running".to_string());
        session.title = Some("Agent A".to_string());
        session.summary = Some("Reviewing session state".to_string());

        let buffer = render_overview_buffer(
            &build_overview_rows(&[session]),
            SortMode::Runtime,
            Rect::new(0, 0, 140, 12),
        );
        let rendered = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("Local Session Overview [runtime]"));
        assert!(rendered.contains("Keys: [s] sort  [r] refresh  [d] diagnostics:off  [q] quit"));
        assert!(rendered.contains("codex"));
        assert!(rendered.contains("codex-cli"));
        assert!(rendered.contains("running"));
        assert!(rendered.contains("Agent A"));
        assert!(rendered.contains("Reviewing"));
    }

    #[test]
    fn diagnostic_overview_buffer_contains_diagnostic_columns() {
        let mut session = Session::minimal("session-1");
        session.cwd = Some("/Users/rj/Code/aisess".to_string());
        session.match_reason = Some("process-cwd".to_string());
        session.dedupe_key = Some("/users/rj/code/aisess".to_string());
        session.hidden_duplicates = 1;

        let buffer = render_diagnostic_overview_buffer(
            &build_overview_rows(&[session]),
            SortMode::Runtime,
            Rect::new(0, 0, 180, 12),
        );
        let rendered = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("[diagnostics]"));
        assert!(rendered.contains("Keys: [s] sort  [r] refresh  [d] diagnostics:on  [q] quit"));
        assert!(rendered.contains("CWD"));
        assert!(rendered.contains("Match"));
        assert!(rendered.contains("process-cwd"));
        assert!(rendered.contains("hidden_dupes:1"));
    }
}
