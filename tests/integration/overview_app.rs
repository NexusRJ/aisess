use aisess::discovery::{normalize_session, RawSessionMetadata};
use aisess::app::App;
use aisess::discovery::{DiscoveryError, SessionDiscovery};
use aisess::sort::{sort_sessions, SortMode};
use aisess::tui::{build_overview_rows, render_overview_text};
use time::macros::datetime;

struct SequenceDiscovery {
    sessions: Result<Vec<aisess::model::Session>, DiscoveryError>,
}

impl SessionDiscovery for SequenceDiscovery {
    fn discover(&self) -> Result<Vec<aisess::model::Session>, DiscoveryError> {
        self.sessions.clone()
    }
}

#[test]
fn renders_multiple_discovered_sessions_in_overview_text() {
    let sessions = vec![
        normalize_session(RawSessionMetadata {
            session_id: "session-1".to_string(),
            provider: Some("codex".to_string()),
            title: Some("Session One".to_string()),
            summary: Some("Tracking feature plan".to_string()),
            runtime_seconds: Some(120),
            token_usage: Some(1000),
            context_length: Some(2048),
            message_count: Some(6),
        }),
        normalize_session(RawSessionMetadata {
            session_id: "session-2".to_string(),
            provider: Some("claude-code".to_string()),
            title: Some("Session Two".to_string()),
            summary: Some("Reviewing state".to_string()),
            runtime_seconds: Some(240),
            token_usage: Some(2000),
            context_length: Some(4096),
            message_count: Some(12),
        }),
    ];

    let rendered = render_overview_text(&build_overview_rows(&sessions));

    assert!(rendered.contains("Session One | Tracking feature plan"));
    assert!(rendered.contains("Session Two | Reviewing state"));
}

#[test]
fn sort_mode_changes_overview_order() {
    let sessions = vec![
        normalize_session(RawSessionMetadata {
            session_id: "session-1".to_string(),
            provider: Some("codex".to_string()),
            title: Some("Session One".to_string()),
            summary: Some("Tracking feature plan".to_string()),
            runtime_seconds: Some(120),
            token_usage: Some(1000),
            context_length: Some(2048),
            message_count: Some(6),
        }),
        normalize_session(RawSessionMetadata {
            session_id: "session-2".to_string(),
            provider: Some("claude-code".to_string()),
            title: Some("Session Two".to_string()),
            summary: Some("Reviewing state".to_string()),
            runtime_seconds: Some(240),
            token_usage: Some(2000),
            context_length: Some(4096),
            message_count: Some(12),
        }),
    ];

    let sorted = sort_sessions(&sessions, SortMode::TokenUsage);
    let rendered = render_overview_text(&build_overview_rows(&sorted));

    let first = rendered.lines().next().unwrap_or_default();
    assert!(first.contains("Session Two | Reviewing state"));
}

#[test]
fn refresh_updates_overview_after_new_snapshot() {
    let mut app = App::new();
    let discovery = SequenceDiscovery {
        sessions: Ok(vec![normalize_session(RawSessionMetadata {
            session_id: "session-3".to_string(),
            provider: Some("codex".to_string()),
            title: Some("Session Three".to_string()),
            summary: Some("Watching refresh".to_string()),
            runtime_seconds: Some(30),
            token_usage: Some(300),
            context_length: Some(1024),
            message_count: Some(3),
        })]),
    };

    app.refresh_with(&discovery, datetime!(2026-05-25 10:10:00 UTC));

    let rendered = render_overview_text(&build_overview_rows(&app.sorted_sessions()));

    assert!(rendered.contains("Session Three | Watching refresh"));
}
