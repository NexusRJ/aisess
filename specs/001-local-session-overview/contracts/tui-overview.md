# TUI Overview Contract

## Purpose
Defines the user-visible contract for the local session overview terminal interface.

## Inputs
- Discovered local session metadata from supported providers.
- User keyboard input for navigation, sort-mode selection, and quitting.
- Timer-driven refresh ticks.

## Required overview fields per row
- Provider label when available
- Session title
- Short summary
- Runtime
- Token usage
- Context length
- Message count
- Metadata availability/state indicator when fields are partial or stale

## Required behaviors
- The overview must render all discovered active sessions in a single list.
- The overview must support ordering by summary, runtime, token usage, and context length.
- The overview must refresh automatically on a configured interval.
- The overview must remain read-only.
- Missing metadata must be displayed as unavailable rather than misleading zero-equivalent values.

## Minimum keyboard interactions
- Change active sort mode
- Trigger a manual refresh if implemented in addition to timed refresh
- Quit the application

## Out of scope for this contract
- Session control actions
- Remote discovery
- Historical log inspection
