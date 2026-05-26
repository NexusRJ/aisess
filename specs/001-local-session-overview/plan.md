# Implementation Plan: Local Session Overview

**Branch**: `001-local-session-overview` | **Date**: 2026-05-25 | **Spec**: `specs/001-local-session-overview/spec.md`

**Input**: Feature specification from `/specs/001-local-session-overview/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Build a local read-only TUI in Rust that periodically discovers and displays all supported open code-agent sessions on the current machine. The implementation uses a normalized adapter layer for heterogeneous provider metadata, a polling refresh loop, and a `ratatui` + `crossterm` interface optimized for dense session overviews and sorting by summary, runtime, token usage, and context length.

## Technical Context

**Language/Version**: Rust stable

**Primary Dependencies**: `ratatui`, `crossterm`, `serde`

**Storage**: N/A

**Testing**: `cargo test`

**Target Platform**: Local desktop terminal environments on the current machine

**Project Type**: Single-binary terminal application

**Performance Goals**: Render the initial overview within 5 seconds and complete refresh-driven updates within one polling interval

**Constraints**: Offline/local-only, read-only session access, TDD required, must follow project-default Rust + `ratatui` stack, must run from a dedicated feature branch kept synchronized with `main` and `dev` as applicable

**Scale/Scope**: Single-user local monitoring of multiple simultaneously open agent sessions on one machine

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Spec-first path confirmed: `specs/001-local-session-overview/spec.md` exists and this plan remains within its scope.
- Simplicity is justified: v1 uses a polling-based read-only adapter model instead of remote coordination or control actions.
- TDD is enforced: unit tests will be written first for normalization, sorting, and refresh-state logic before implementation.
- Branch workflow is defined: work is on `001-local-session-overview`, and the branch must be synchronized with `main` and `dev` before major implementation milestones when `dev` exists.
- Project stack is respected: this feature uses the default Rust + `ratatui` + `crossterm` direction without exception.

## Project Structure

### Documentation (this feature)

```text
specs/001-local-session-overview/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── tui-overview.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── app/
├── discovery/
├── model/
├── sort/
└── tui/

tests/
└── integration/
```

**Structure Decision**: Use a single Rust terminal application with separate modules for provider discovery, normalized domain state, sorting/refresh logic, and TUI rendering. Keep provider-specific session parsing behind discovery adapters so UI code stays provider-neutral.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
