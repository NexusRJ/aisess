# Tasks: Local Session Overview

**Input**: Design documents from `/specs/001-local-session-overview/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: This feature uses TDD. For each user story, write the listed tests first, confirm they fail, then implement the smallest change to make them pass.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize the Rust application structure and TDD-ready test scaffolding

- [ ] T001 Create Rust application structure and module entry points in `Cargo.toml`, `src/main.rs`, `src/app/mod.rs`, `src/discovery/mod.rs`, `src/model/mod.rs`, `src/sort/mod.rs`, and `src/tui/mod.rs`
- [ ] T002 Configure core dependencies for the TUI application in `Cargo.toml`
- [ ] T003 [P] Create integration test scaffold for end-to-end overview flows in `tests/integration/overview_app.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish the normalized session model, refresh flow, and provider adapter boundaries used by all stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 [P] Create failing unit tests for normalized session parsing and missing-metadata handling in `src/model/mod.rs`
- [ ] T005 [P] Create failing unit tests for overview refresh-state transitions in `src/app/mod.rs`
- [ ] T006 Implement the normalized session and overview snapshot models in `src/model/mod.rs`
- [ ] T007 Implement application refresh state and polling coordination in `src/app/mod.rs`
- [ ] T008 Define provider discovery adapter interfaces and shared metadata error handling in `src/discovery/mod.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Review all active sessions at a glance (Priority: P1) 🎯 MVP

**Goal**: Show all discovered local sessions in one read-only TUI overview with the required summary fields

**Independent Test**: Launch the app against fixture-backed local session metadata and verify that all discovered sessions appear with title, summary, runtime, token usage, context length, and message count when available.

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T009 [P] [US1] Create failing unit tests for provider metadata normalization into overview rows in `src/discovery/mod.rs`
- [ ] T010 [P] [US1] Create failing unit tests for row formatting with missing fields in `src/tui/mod.rs`
- [ ] T011 [US1] Create failing integration test for rendering multiple discovered sessions in `tests/integration/overview_app.rs`

### Implementation for User Story 1

- [ ] T012 [US1] Implement the first local provider discovery adapter and metadata normalization pipeline in `src/discovery/mod.rs`
- [ ] T013 [US1] Implement overview row view models for title, summary, runtime, token usage, context length, and message count in `src/tui/mod.rs`
- [ ] T014 [US1] Implement the read-only session overview screen in `src/tui/mod.rs` and wire it through `src/main.rs`
- [ ] T015 [US1] Validate the MVP flow against the independent test and keep the new test suite green

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Re-rank sessions by the most useful signals (Priority: P2)

**Goal**: Let the user reorder the overview by summary, runtime, token usage, and context length

**Independent Test**: Run the app with sessions containing distinct summary and metric values, switch sort modes, and verify the displayed order changes consistently for each supported mode.

### Tests for User Story 2 ⚠️

- [ ] T016 [P] [US2] Create failing unit tests for supported sort modes and deterministic missing-value handling in `src/sort/mod.rs`
- [ ] T017 [US2] Create failing integration test for changing sort modes from the TUI in `tests/integration/overview_app.rs`

### Implementation for User Story 2

- [ ] T018 [US2] Implement overview sorting by summary, runtime, token usage, and context length in `src/sort/mod.rs`
- [ ] T019 [US2] Implement read-only TUI controls and state updates for switching sort modes in `src/tui/mod.rs`
- [ ] T020 [US2] Integrate sorting behavior into the app state and refresh flow in `src/app/mod.rs`
- [ ] T021 [US2] Validate sorting behavior against the independent test and keep the test suite green

**Checkpoint**: At this point, User Stories 1 and 2 should both work independently

---

## Phase 5: User Story 3 - Keep the overview current without manual restarts (Priority: P3)

**Goal**: Refresh the overview automatically so session additions, removals, and metric changes appear without restarting the app

**Independent Test**: Run the app with mutable fixture-backed session data, wait for refresh intervals, and verify that additions, removals, and metric updates appear automatically.

### Tests for User Story 3 ⚠️

- [ ] T022 [P] [US3] Create failing unit tests for polling refresh updates, including added, removed, and changed sessions in `src/app/mod.rs`
- [ ] T023 [US3] Create failing integration test for timed refresh behavior in `tests/integration/overview_app.rs`

### Implementation for User Story 3

- [ ] T024 [US3] Implement timed polling refresh and snapshot replacement in `src/app/mod.rs`
- [ ] T025 [US3] Implement TUI update handling for refreshed session lists and metrics in `src/tui/mod.rs`
- [ ] T026 [US3] Harden refresh behavior for partial or temporarily unreadable metadata in `src/discovery/mod.rs` and `src/app/mod.rs`
- [ ] T027 [US3] Validate timed refresh behavior against the independent test and keep the test suite green

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finalize usability, documentation, and regression confidence without expanding scope

- [ ] T028 [P] Add quickstart-aligned run instructions and developer notes in `specs/001-local-session-overview/quickstart.md` and project docs as needed
- [ ] T029 Run the full Rust test suite with `cargo test` and resolve only feature-related failures
- [ ] T030 Perform manual validation of the TUI overview against the checklist in `specs/001-local-session-overview/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - blocks all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational completion
- **User Story 2 (Phase 4)**: Depends on User Story 1 overview structures being available
- **User Story 3 (Phase 5)**: Depends on Foundational completion and benefits from User Story 1 integration paths
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: First deliverable and MVP
- **User Story 2 (P2)**: Depends on the overview list from User Story 1
- **User Story 3 (P3)**: Depends on the refreshable application state established in Foundational work and overview rendering from User Story 1

### Within Each User Story

- Tests MUST be written and fail before implementation
- Domain/state logic before TUI integration where possible
- Provider discovery before end-to-end rendering for affected stories
- Story validation before moving to the next priority

### Parallel Opportunities

- T003 can run in parallel with T001-T002
- T004 and T005 can run in parallel
- T009 and T010 can run in parallel
- T016 can run in parallel with T017 setup work
- T022 can run in parallel with T023 setup work

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Confirm the overview lists all discovered local sessions with required fields

### Incremental Delivery

1. Complete Setup + Foundational → session model and refresh framework ready
2. Add User Story 1 → validate unified overview (MVP)
3. Add User Story 2 → validate sorting behavior
4. Add User Story 3 → validate timed refresh behavior
5. Finish with documentation and full regression pass

## Notes

- Keep scope focused on local read-only monitoring
- Prefer tests over implementation detail when clarifying behavior
- Avoid adding remote aggregation, session control, or historical log parsing in this feature
- Commit after each completed story or logical milestone
