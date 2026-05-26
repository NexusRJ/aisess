# Data Model: Local Session Overview

## Session
- Purpose: Represents one discovered local code-agent session in normalized form.
- Fields:
  - `session_id`: Stable identifier from the provider or derived local identity.
  - `provider`: Source agent type, such as Codex or Claude Code.
  - `title`: Primary session title when available.
  - `summary`: Short readable summary derived from metadata.
  - `status`: Current provider-reported state if available.
  - `started_at`: Session start time when available.
  - `last_active_at`: Most recent activity timestamp when available.
  - `runtime`: Computed elapsed runtime for display and sorting.
  - `token_usage`: Aggregate token usage when available.
  - `context_length`: Current context size when available.
  - `message_count`: Number of messages/items in the current context when available.
  - `metadata_state`: Availability marker describing whether metadata is complete, partial, stale, or unreadable.
- Validation rules:
  - `session_id` must be unique within an overview snapshot.
  - `runtime`, `token_usage`, `context_length`, and `message_count` must be non-negative when present.
  - Missing metrics remain optional and must not be coerced into misleading zero values.

## Overview Snapshot
- Purpose: Represents the full discovered session state at one refresh point.
- Fields:
  - `captured_at`: Time the refresh completed.
  - `sessions`: Collection of normalized `Session` records.
  - `refresh_status`: Whether the refresh succeeded fully, partially, or failed.
  - `errors`: Non-fatal adapter or discovery errors captured during refresh.
- Validation rules:
  - A snapshot may contain zero sessions.
  - Partial provider failures must not invalidate otherwise valid session records.

## Sort Mode
- Purpose: Represents the active ordering applied to the overview.
- Values:
  - `summary`
  - `runtime`
  - `token_usage`
  - `context_length`
- Validation rules:
  - Exactly one sort mode is active at a time.
  - Sorting must define deterministic fallback behavior for missing values.

## Refresh Configuration
- Purpose: Captures runtime refresh behavior.
- Fields:
  - `interval_seconds`: Configured polling interval.
  - `last_refresh_started_at`: Most recent refresh start timestamp.
  - `last_refresh_completed_at`: Most recent refresh completion timestamp.
- Validation rules:
  - `interval_seconds` must be greater than zero.
  - Completion timestamps cannot precede start timestamps.
