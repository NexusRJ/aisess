# AISess Constitution

## Core Principles

### I. Spec-First Delivery
Every non-trivial change MUST begin with a written specification before implementation starts. Work proceeds in the order `spec -> plan -> tasks -> implementation`, and each artifact MUST remain consistent with the one before it. If scope changes during execution, the relevant specification artifacts MUST be updated before code work continues.

### II. Simplicity Over Premature Abstraction
Choose the simplest design that satisfies the current requirement. New layers, frameworks, abstractions, or infrastructure are allowed only when justified by a concrete need documented in the plan. Avoid speculative extensibility, avoid broad refactors unrelated to the active task, and prefer localized, reversible changes.

### III. Test-Driven Development (TDD)
All implementation work MUST follow TDD. Developers MUST first write or update a failing unit test that captures the intended behavior, then implement the smallest production change needed to make that test pass, and finally refactor while keeping the test suite green. During development, the relevant tests MUST be rerun frequently enough to catch regressions immediately. Production code changes without corresponding preceding tests are non-compliant except for purely non-executable assets such as prose documentation or configuration with no executable behavior.

## Project Constraints

- Primary workflow files live under `specs/` for feature work and `.specify/` for shared process assets.
- All new feature work SHOULD use the provided Spec Kit templates unless there is a documented reason to diverge.
- Git is the source of truth for progress tracking, reviewability, and milestone capture.
- Documentation should be concise, implementation-facing, and kept in sync with real behavior.

## Development Workflow

1. Establish or refine the governing constitution when project rules change.
2. Create a feature specification before implementation.
3. Produce an implementation plan that passes the Constitution Check.
4. Generate tasks that preserve clear execution order and validation steps.
5. Implement in small batches, validating each batch before declaring completion.
6. Record meaningful milestones in Git with focused commits.

## Governance

This constitution governs project workflow and overrides conflicting informal practice. Amendments require documenting the change, updating any affected templates, and recording the change in version metadata below.

Versioning rules:
- MAJOR: Removes or materially redefines a governing principle.
- MINOR: Adds a new principle or materially expands workflow requirements.
- PATCH: Clarifies wording, fixes placeholders, or improves guidance without changing intent.

Compliance review expectations:
- Every plan MUST include a Constitution Check against these principles.
- Implementation plans MUST justify any new abstraction beyond the simplest viable design.
- Implementation tasks MUST reflect TDD order: failing test first, implementation second, refactor third.
- Implementation reviews SHOULD reject work that skips required spec artifacts or violates TDD.

**Version**: 1.0.0 | **Ratified**: 2026-05-24 | **Last Amended**: 2026-05-24
