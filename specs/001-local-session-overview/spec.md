# Feature Specification: Local Session Overview

**Feature Branch**: `001-local-session-overview`

**Created**: 2026-05-25

**Status**: Draft

**Input**: User description: "Build a local read-only TUI that periodically refreshes and shows all open code agent sessions on the current machine, including title, short summary, runtime, token usage, context length, and message count."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Review all active sessions at a glance (Priority: P1)

As an individual user running many code-agent sessions on the current machine, I want to see all discovered open sessions in one TUI view so that I can quickly understand what is running and where my attention is needed.

**Why this priority**: This is the core value of the feature. Without a unified overview of all local sessions, the tool does not solve the main coordination problem.

**Independent Test**: Can be fully tested by opening multiple local sessions with distinct titles and metadata, launching the TUI, and verifying that the list includes each discovered session with the expected summary fields.

**Acceptance Scenarios**:

1. **Given** multiple supported code-agent sessions are open on the local machine, **When** the user opens the TUI, **Then** the system shows all discovered sessions in a single overview list.
2. **Given** a discovered session has a title and metadata summary available, **When** the session is displayed in the overview, **Then** the row includes the title and a short readable summary.
3. **Given** a discovered session has runtime, token usage, context length, and message-count metadata available, **When** the session is displayed, **Then** those values are shown alongside the session summary.

---

### User Story 2 - Re-rank sessions by the most useful signals (Priority: P2)

As an individual user managing many concurrent sessions, I want to sort the overview by topic summary, runtime, token usage, and context size so that I can quickly find the most important, longest-running, or most resource-intensive sessions.

**Why this priority**: Once the sessions are visible, sorting is the fastest way to make the overview practically useful during active work.

**Independent Test**: Can be tested independently by loading the overview with sessions that have different summaries and numeric metrics, applying each supported sort mode, and verifying the displayed order updates correctly.

**Acceptance Scenarios**:

1. **Given** the overview contains sessions with different runtimes, **When** the user selects runtime sorting, **Then** the list reorders by runtime consistently.
2. **Given** the overview contains sessions with different token usage or context length values, **When** the user selects those sort modes, **Then** the list reorders by the chosen metric.
3. **Given** the overview contains sessions with different titles or summaries, **When** the user selects the topic-based sort mode, **Then** the list reorders consistently using the displayed text fields.

---

### User Story 3 - Keep the overview current without manual restarts (Priority: P3)

As an individual user monitoring local sessions during active work, I want the TUI to refresh on a timed interval so that newly opened sessions, closed sessions, and changing usage values appear without restarting the tool.

**Why this priority**: Automatic refresh makes the overview reliable during long-running work sessions, but the core value still exists without it.

**Independent Test**: Can be tested independently by starting the TUI, changing the set of open sessions or their metadata, waiting for the refresh interval, and verifying the view updates automatically.

**Acceptance Scenarios**:

1. **Given** the TUI is running and a new supported local session starts, **When** the next refresh cycle completes, **Then** the new session appears in the overview.
2. **Given** the TUI is running and an existing session closes, **When** the next refresh cycle completes, **Then** the closed session no longer appears as active.
3. **Given** session usage metrics change over time, **When** the next refresh cycle completes, **Then** the displayed runtime, token usage, context length, and message count reflect the latest discovered metadata.

### Edge Cases

- What happens when no supported sessions are currently open on the machine?
- How does the system handle sessions whose metadata is partially missing, stale, or temporarily unreadable?
- How does the system handle unusually long titles or summaries that exceed the available TUI width?
- What happens when two or more sessions have identical titles or near-identical summaries?
- How does the system behave when metadata changes during an active refresh cycle?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST discover supported open code-agent sessions on the current local machine without requiring remote connectivity.
- **FR-002**: System MUST present all discovered active sessions in a single read-only TUI overview.
- **FR-003**: System MUST display a title for each session when a title is available from current metadata.
- **FR-004**: System MUST display a short readable summary for each session derived from current metadata when available.
- **FR-005**: System MUST display each session's runtime using current metadata.
- **FR-006**: System MUST display each session's token usage using current metadata when available.
- **FR-007**: System MUST display each session's context length and message count for each discovered session when available.
- **FR-008**: System MUST clearly indicate when a session field is unavailable rather than showing misleading placeholder values.
- **FR-009**: System MUST support timed automatic refresh of the overview without requiring the user to restart the application.
- **FR-010**: Users MUST be able to sort the overview by topic summary, runtime, token usage, and context length.
- **FR-011**: System MUST keep the feature read-only and MUST NOT send commands, modify sessions, or alter agent state.
- **FR-012**: System MUST update the overview to reflect newly discovered sessions and sessions that are no longer active after refresh.
- **FR-013**: System MUST continue operating when metadata for one or more sessions is incomplete or temporarily unreadable.
- **FR-014**: System MUST identify the source agent type for each session when that information is available from current metadata.

### Key Entities *(include if feature involves data)*

- **Session**: A currently open local code-agent session, including identifying metadata, title, summary, activity timing, usage metrics, and source agent type.
- **Session Metrics**: The usage and size values associated with a session, including runtime, token usage, context length, and message count.
- **Overview Snapshot**: A read-only view of all discovered local sessions at a specific refresh point.
- **Sort Mode**: The user-selected ordering mode applied to the overview list.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can view all discovered active local sessions in a single overview within 5 seconds of opening the tool.
- **SC-002**: After a refresh cycle completes, session additions, removals, and metric changes are reflected in the overview within one refresh interval.
- **SC-003**: Users can reorder the same session set by any supported sort mode in no more than 2 interactions.
- **SC-004**: In sessions where the source metadata provides title, summary, runtime, token usage, context length, and message count, the overview displays those values for at least 95% of discovered active sessions.

## Assumptions

- The first release targets a single local machine and does not include remote or cross-device session aggregation.
- The first release reads only currently available metadata and does not parse historical logs or conversation transcripts.
- The first release is read-only and excludes actions such as closing sessions, sending input, or resuming agent work.
- The primary user is an individual developer managing many concurrent code-agent sessions.
- The first release uses a TUI as the delivery interface; other surfaces such as web, desktop GUI, or status bar may be explored later.
