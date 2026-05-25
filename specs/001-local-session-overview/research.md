# Research: Local Session Overview

## Decision 1: Use Rust as the implementation language
- Decision: Use stable Rust for the application and supporting libraries.
- Rationale: Rust matches the project's default stack, produces a single distributable binary, and is a strong fit for a local system-oriented tool with predictable runtime behavior.
- Alternatives considered:
  - Python: Faster for prototyping, but weaker for single-binary distribution and long-term systems tooling.
  - Go: Good systems ergonomics, but weaker TUI ecosystem fit for the desired interface compared with the chosen stack.

## Decision 2: Use `ratatui` with `crossterm` for the TUI
- Decision: Build the interactive terminal interface with `ratatui` and use `crossterm` as the terminal backend.
- Rationale: This stack is the project default, fits dense tabular/status layouts well, and supports the keyboard-driven read-only monitoring workflow required by the feature.
- Alternatives considered:
  - tui-rs legacy stack: Superseded by ratatui and less aligned with current community maintenance.
  - Building a raw terminal renderer: Simpler dependency graph, but too costly for layout, widgets, and maintainability.

## Decision 3: Use polling-based refresh over event-driven discovery for v1
- Decision: Refresh session data on a fixed interval rather than relying on provider-specific live event streams.
- Rationale: The feature is read-only, local-only, and metadata-based. Polling minimizes integration complexity and works consistently across heterogeneous agent sources.
- Alternatives considered:
  - File watchers: Useful if metadata is file-backed, but not universal enough for all providers.
  - Event subscriptions: Better latency, but too provider-specific for a first version.

## Decision 4: Normalize provider metadata through a shared adapter model
- Decision: Define a provider-neutral session model and map each supported agent source into it through adapters.
- Rationale: The major risk in this feature is metadata heterogeneity. A normalized model isolates that complexity and keeps sorting/rendering logic independent of any one provider.
- Alternatives considered:
  - Provider-specific rendering paths: Faster initially, but would complicate sorting, refresh, and future provider expansion.
  - Persisting raw metadata directly in the UI state: Simpler short-term, but too brittle for missing or evolving fields.

## Decision 5: Prefer unit tests on transformation, sorting, and refresh-state logic
- Decision: Apply TDD primarily to metadata normalization, sorting behavior, and refresh-state transitions.
- Rationale: These areas carry the most logic and are stable to test without binding tests tightly to terminal rendering details.
- Alternatives considered:
  - Snapshot-heavy UI tests: Possible, but lower value as a first validation layer.
  - Manual-only verification: Rejected due to the project's TDD rule.

## Open integration note: Remote monitoring
- Decision: Keep remote monitoring out of scope for v1 and document it as a future extension.
- Rationale: Supporting remote sessions would require an explicit transport, discovery protocol, and trust model not needed for the initial local overview.
- Alternatives considered:
  - SSH-based scraping: Too environment-specific.
  - Shared network service: Better long-term direction, but premature for the current feature.
