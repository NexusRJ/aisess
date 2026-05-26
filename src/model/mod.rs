use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataState {
    Complete,
    Partial,
    Stale,
    Unreadable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub provider: Option<String>,
    pub source: Option<String>,
    pub activity_state: Option<String>,
    pub cwd: Option<String>,
    pub match_reason: Option<String>,
    pub dedupe_key: Option<String>,
    pub hidden_duplicates: u64,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub status: Option<String>,
    pub started_at: Option<OffsetDateTime>,
    pub last_active_at: Option<OffsetDateTime>,
    pub runtime_seconds: Option<u64>,
    pub token_usage: Option<u64>,
    pub context_length: Option<u64>,
    pub message_count: Option<u64>,
    pub metadata_state: MetadataState,
}

impl Session {
    pub fn minimal(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            provider: None,
            source: None,
            activity_state: None,
            cwd: None,
            match_reason: None,
            dedupe_key: None,
            hidden_duplicates: 0,
            title: None,
            summary: None,
            status: None,
            started_at: None,
            last_active_at: None,
            runtime_seconds: None,
            token_usage: None,
            context_length: None,
            message_count: None,
            metadata_state: MetadataState::Partial,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewSnapshot {
    pub captured_at: OffsetDateTime,
    pub sessions: Vec<Session>,
    pub errors: Vec<String>,
}

impl OverviewSnapshot {
    pub fn empty() -> Self {
        Self {
            captured_at: OffsetDateTime::UNIX_EPOCH,
            sessions: vec![],
            errors: vec![],
        }
    }

    pub fn provider_count(&self, provider: &str) -> usize {
        self.sessions
            .iter()
            .filter(|session| session.provider.as_deref() == Some(provider))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::{MetadataState, Session};

    #[test]
    fn minimal_session_defaults_to_partial_metadata() {
        let session = Session::minimal("session-1");

        assert_eq!(session.session_id, "session-1");
        assert_eq!(session.metadata_state, MetadataState::Partial);
        assert!(session.runtime_seconds.is_none());
        assert!(session.token_usage.is_none());
    }
}
