# Progress Log: Local Session Overview

## Current Status

Feature implementation is partially complete and currently in an active debugging/tuning phase focused on session detection accuracy.

### Working Areas
- Rust project scaffold is in place.
- TUI event loop is runnable with `ratatui` + `crossterm`.
- Sorting is implemented for summary, runtime, token usage, and context length.
- Refresh loop skeleton is implemented.
- Real local providers exist for:
  - Codex sessions under `~/.codex/sessions`
  - Claude sessions under `~/.claude/projects`
- UI currently shows:
  - `Provider`
  - `Source`
  - `State`
  - `Title`
  - `Summary`
  - `Runtime`
  - `Tokens`
  - `Context`
  - `Msgs`
- Status bar currently shows:
  - active count
  - claude count
  - codex count
  - current sort mode
  - last refresh timestamp

### Validation Status
- `cargo test` currently passes.
- Last known passing state during this session: `26 passed, 0 failed`.

## Implemented Detection Behavior

### Codex
- Reads `.jsonl` session files from `~/.codex/sessions`
- Extracts session metadata, summary, runtime, token/context hints when available
- Distinguishes `source` values such as:
  - `codex-cli`
  - `codex-desktop`
  - `codex-unknown`
- Marks `state` as:
  - `running` when CLI-process-assisted matching succeeds
  - `recent` when session is only matched by recent log activity
- Uses process inspection via `ps` and `lsof` to assist CLI activity detection
- Applies partial de-duplication for same-project Codex sessions

### Claude
- Reads `.jsonl` session files from `~/.claude/projects`
- Extracts summary, runtime, message count, and some token information when available
- Marks `source` as `claude-local`
- Marks `state` as `recent`, and code has started to move toward process-assisted matching

## Known Problems Still Not Fully Solved

### 1. Codex duplicate sessions for same project
Observed behavior:
- Multiple sessions from the same project, especially `power`, can still appear.
- Example seen during debugging:
  - one `power` entry with long runtime (likely real current shell tab)
  - one `power` entry with short runtime (likely old historical session)

Current understanding:
- Current de-duplication is not yet strict enough.
- Same-project historical sessions can still survive if matching is not collapsed hard enough.

Desired behavior:
- For the same active CLI project, only show the single best current session.
- Older same-project Codex sessions should not be shown.

### 2. Claude Code session still may not appear
Observed behavior:
- User reports there is one active Claude Code session, but it still may not show.

Current understanding:
- Claude matching is still weaker than Codex matching.
- It likely needs stronger use of actual `cwd` from JSONL entries and better process-to-session matching.

Desired behavior:
- Active Claude Code shell session should show similarly to active Codex CLI sessions.

### 3. Source classification still needs verification
Observed behavior:
- Codex desktop and CLI sessions are better separated than before, but still need real-world verification after each tuning pass.

Desired behavior:
- Desktop-generated sessions should not be mislabeled as `codex-cli running`.
- CLI sessions should be clearly distinguishable from desktop sessions.

## Recommended Next Debug Steps

### Next Step 1: Harden Codex same-project deduplication
- Restrict same-project `codex-cli running` to exactly one session
- Prefer the candidate with the strongest current-process match and most recent activity
- Hide older same-project sessions instead of downgrading them into visible duplicates

### Next Step 2: Strengthen Claude process-assisted matching
- Prefer real `cwd` parsed from Claude JSONL entries
- If missing, fall back to project-directory decoding from `.claude/projects/...`
- Match Claude shell process `cwd` against parsed session `cwd`

### Next Step 3: Add debugging visibility if needed
If detection is still unclear after the above:
- temporarily add a debug column or status detail for:
  - matched cwd
  - provider raw source hint
  - whether match came from process or recent log activity

## Important Notes For Resume
- Current branch: `001-local-session-overview`
- Work is not fully complete even though tests pass.
- The biggest remaining gap is detection accuracy, not base TUI functionality.
- The most recent user-approved direction is:
  1. Hide old same-project Codex sessions directly
  2. Improve Claude matching using real `cwd`

## Suggested Resume Prompt
If resuming later, use something like:

`Continue 001-local-session-overview. Read progress.md first. Focus on removing duplicate same-project Codex sessions and making the active Claude Code session appear by improving cwd-based matching.`

## 2026-05-25 Update: CWD Matching and Same-Project Deduplication

### Completed This Pass
- Added normalized `cwd` storage to `Session` so provider discovery can match against real working directories instead of relying on display titles.
- Codex parsing now preserves `cwd` from `session_meta` / `turn_context` and uses exact normalized cwd comparison before project-name fallback.
- Codex deduplication now collapses running same-project sessions by project key and keeps only the most recently active matching session, hiding older same-project duplicates.
- Claude parsing now preserves real `cwd` from JSONL entries when present and falls back to decoding `.claude/projects/...` directory names into a cwd.
- Claude process matching now compares process cwd to session cwd before falling back to project-name matching.

### Regression Coverage Added
- Codex duplicate running same-project sessions collapse to the newest session.
- Claude JSONL cwd is retained and matches a live Claude process cwd.
- Codex discovered sessions expose parsed cwd.

### Validation
- `cargo test` passes: 28 passed, 0 failed.
- `cargo fmt --check` was attempted but reports formatting drift in pre-existing untracked files outside the focused change set; only `src/discovery/mod.rs` and `src/model/mod.rs` were formatted for this pass.

### Remaining Verification
- Run the TUI against the real machine state and verify that:
  - only one active Codex row appears per same-project CLI session;
  - the active Claude Code row appears as `claude-code` / `claude-local` with `running` state;
  - Codex desktop sessions are not mislabeled as `codex-cli running`.

## 2026-05-26 Update: Claude Same-Project Running Deduplication

### Completed This Pass
- Added Claude-specific deduplication after discovery so multiple JSONL sessions matching the same active project cwd collapse into one visible row.
- Reused the normalized session project key (`cwd` first, then title fallback) so Claude dedupe behaves like the working Codex same-project collapse.
- Kept the newest `last_active_at` Claude session for each project key, which matches the expected single open Claude Code session behavior.
- Added `CodexDiscovery::with_active_window` to make the Codex fixture test stable as calendar time moves forward.

### Regression Coverage Added
- Claude duplicate running same-project sessions collapse to the newest session.
- Existing Codex fixture discovery remains stable under an explicit active-window override.

### Validation
- `cargo test discovery --lib` passes: 12 passed, 0 failed.
- `cargo test` passes: 29 passed, 0 failed.

### Remaining Verification
- Run the TUI against the real machine state and confirm Claude Code now shows only one running row for the single open project/session.

### Correction
- Fixed an intermediate patch mistake so Claude deduplication is applied inside `ClaudeDiscovery` only, not globally in `CombinedDiscovery`; this prevents cross-provider rows from being collapsed together.

## 2026-05-26 Update: TUI Diagnostic Mode

### Completed This Pass
- Added diagnostic metadata to normalized sessions: parsed cwd, match reason, dedupe key, and hidden duplicate count.
- Discovery now marks running sessions with `process-cwd` or `project-name`, and recent-only sessions with `recent-log`.
- Deduplication now records the key used to keep a row and increments `hidden_duplicates` on the retained row.
- Added `d` key support to toggle diagnostic mode in the TUI.
- Diagnostic mode adds CWD, Match, Key, and Hidden columns while keeping the default overview unchanged.
- Status bar now shows whether diagnostics are on and the total hidden duplicate count.

### Regression Coverage Added
- App diagnostic toggle state switches on/off.
- TUI diagnostic rows preserve cwd/match/key/hidden details.
- Diagnostic rendering includes diagnostic title/columns and hidden duplicate totals.

### Validation
- `cargo test` passes: 32 passed, 0 failed.

## 2026-05-26 Final Closeout

### Completed This Pass
- Revalidated the current feature branch state after the interrupted session and confirmed the implemented overview, sorting, refresh, deduplication, and diagnostics flows are all present.
- Ran `cargo fmt` across the Rust workspace so the source tree now satisfies the formatter check.
- Refreshed the quickstart to document the actual keyboard controls now implemented in the TUI, including sort cycling, manual refresh, diagnostics toggle, and quit.
- Updated this progress log to capture the final validation state for handoff.

### Validation
- `cargo test` passes: 32 passed, 0 failed.
- `cargo fmt --check` passes.

### Remaining Verification
- Manual on-machine TUI verification is still recommended for the real local session environment, especially provider matching accuracy against live Codex and Claude sessions.

## 2026-05-26 Update: Visible Key Hints

### Completed This Pass
- Promoted the keyboard help from a low-contrast table border caption into a dedicated visible help bar below the overview table.
- The help bar now shows `s`, `r`, `d`, and `q` controls in one place and reflects whether diagnostics are currently on or off.
- Added a regression test that verifies the help line content is rendered into the overview buffer.

### Validation
- `cargo fmt --check` passes.
- `cargo test --quiet` passes: 33 passed, 0 failed.

### Remaining Verification
- Include the new help bar in the manual acceptance pass and confirm it remains readable in the target terminal theme.


## 2026-05-26 Manual Acceptance

### Result
- Manual acceptance passed in the real local session environment.

### Verified
- The TUI launched and exited cleanly.
- Active supported sessions appeared as expected.
- Required overview fields rendered correctly, with unavailable values shown safely.
- Sorting, manual refresh, automatic refresh, and diagnostic toggle all behaved as expected.
- Visible key hints were readable and sufficient during use.
- Codex and Claude matching / deduplication behavior matched the expected live session state.
