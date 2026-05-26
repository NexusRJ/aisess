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
2. Run `cargo run` to launch the TUI application.
3. Confirm that both sessions appear in the overview with title, summary, runtime, token usage, context length, and message count when available.
4. Press `s` to cycle sort mode and confirm the ordering updates.
5. Press `r` to trigger a manual refresh and confirm the status line updates.
6. Press `d` to toggle diagnostic mode and confirm CWD / match / dedupe details appear and disappear.
7. Open or close a session and verify the overview updates after the next refresh interval.
8. Press `q` to exit cleanly.

## Recommended acceptance flow
1. In terminal A, enter the repo and run `cargo run`.
2. Keep terminal A open with the TUI visible during the whole check.
3. In terminal B, open or keep one active Codex CLI session in this repo.
4. If available on this machine, also keep one active Claude Code session open in the same repo or another repo.
5. Back in terminal A, verify whether the overview shows one row per active real session instead of multiple stale duplicates for the same project.
6. Press `d` and inspect the diagnostic columns:
   - `CWD` should match the repo path used by the live session.
   - `Match` should look reasonable, such as `process-cwd`, `project-name`, or `recent-log`.
   - `Hidden` should be greater than zero only when duplicate stale sessions were intentionally collapsed.
7. Press `s` repeatedly and confirm ordering changes across summary, runtime, token usage, and context length.
8. Press `r` to force a refresh and confirm the status line updates without breaking the list.
9. Start or close one supported session in terminal B, wait one polling interval, and confirm the list updates automatically.
10. Press `q` to exit, then record any mismatch between live sessions and rendered rows.

## Acceptance notes to record
- Whether the live Codex CLI session appeared exactly once.
- Whether the live Claude Code session appeared exactly once.
- Whether any desktop or stale historical session was misclassified as currently running.
- Whether the diagnostic `CWD` and `Match` fields explain each visible row.
