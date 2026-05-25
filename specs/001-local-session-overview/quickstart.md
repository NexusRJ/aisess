# Quickstart: Local Session Overview

## Goal
Run the first local TUI prototype that discovers supported sessions, renders them in a unified overview, and refreshes automatically.

## Prerequisites
- Rust toolchain installed
- Local machine with one or more supported code-agent sessions available for discovery

## Expected developer flow
1. Implement a provider adapter that reads current session metadata from one supported local source.
2. Add unit tests first for normalization and sorting behavior.
3. Run `cargo test` and confirm the new tests fail.
4. Implement the smallest change needed to pass the tests.
5. Run `cargo test` again and keep the suite green while refactoring.
6. Launch the TUI locally and verify that active sessions appear and refresh on schedule.

## Manual validation checklist
1. Start at least two supported local sessions with distinguishable titles or summaries.
2. Run the TUI application.
3. Confirm that both sessions appear in the overview with title, summary, runtime, token usage, context length, and message count when available.
4. Change the active sort mode and confirm the ordering updates.
5. Open or close a session and verify the overview updates after the next refresh interval.
