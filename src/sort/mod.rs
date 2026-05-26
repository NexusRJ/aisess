use crate::model::Session;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Summary,
    Runtime,
    TokenUsage,
    ContextLength,
}

impl SortMode {
    pub fn label(self) -> &'static str {
        match self {
            SortMode::Summary => "summary",
            SortMode::Runtime => "runtime",
            SortMode::TokenUsage => "token_usage",
            SortMode::ContextLength => "context_length",
        }
    }
}

pub fn sort_sessions(sessions: &[Session], mode: SortMode) -> Vec<Session> {
    let mut sorted = sessions.to_vec();

    sorted.sort_by(|left, right| match mode {
        SortMode::Summary => compare_optional_text(&left.summary, &right.summary),
        SortMode::Runtime => compare_optional_u64_desc(left.runtime_seconds, right.runtime_seconds),
        SortMode::TokenUsage => compare_optional_u64_desc(left.token_usage, right.token_usage),
        SortMode::ContextLength => {
            compare_optional_u64_desc(left.context_length, right.context_length)
        }
    });

    sorted
}

fn compare_optional_text(left: &Option<String>, right: &Option<String>) -> std::cmp::Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.to_lowercase().cmp(&right.to_lowercase()),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn compare_optional_u64_desc(left: Option<u64>, right: Option<u64>) -> std::cmp::Ordering {
    match (left, right) {
        (Some(left), Some(right)) => right.cmp(&left),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::{SortMode, sort_sessions};
    use crate::model::Session;

    #[test]
    fn sorts_by_summary_with_missing_values_last() {
        let mut alpha = Session::minimal("alpha");
        alpha.summary = Some("Alpha task".to_string());

        let mut gamma = Session::minimal("gamma");
        gamma.summary = Some("Gamma task".to_string());

        let missing = Session::minimal("missing");

        let sorted = sort_sessions(&[gamma, missing, alpha], SortMode::Summary);

        assert_eq!(sorted[0].session_id, "alpha");
        assert_eq!(sorted[1].session_id, "gamma");
        assert_eq!(sorted[2].session_id, "missing");
    }

    #[test]
    fn sorts_runtime_descending() {
        let mut short = Session::minimal("short");
        short.runtime_seconds = Some(10);

        let mut long = Session::minimal("long");
        long.runtime_seconds = Some(50);

        let missing = Session::minimal("missing");

        let sorted = sort_sessions(&[short, missing, long], SortMode::Runtime);

        assert_eq!(sorted[0].session_id, "long");
        assert_eq!(sorted[1].session_id, "short");
        assert_eq!(sorted[2].session_id, "missing");
    }

    #[test]
    fn sorts_token_usage_descending() {
        let mut low = Session::minimal("low");
        low.token_usage = Some(100);

        let mut high = Session::minimal("high");
        high.token_usage = Some(900);

        let sorted = sort_sessions(&[low, high], SortMode::TokenUsage);

        assert_eq!(sorted[0].session_id, "high");
        assert_eq!(sorted[1].session_id, "low");
    }

    #[test]
    fn sorts_context_length_descending() {
        let mut small = Session::minimal("small");
        small.context_length = Some(128);

        let mut large = Session::minimal("large");
        large.context_length = Some(4096);

        let sorted = sort_sessions(&[small, large], SortMode::ContextLength);

        assert_eq!(sorted[0].session_id, "large");
        assert_eq!(sorted[1].session_id, "small");
    }
}
