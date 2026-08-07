# AGENTS.md

## General principles

Follow the existing architecture, conventions, and style of the repository.

Prefer simple, readable, reusable, and maintainable solutions. Avoid unnecessary abstractions and disruptive changes unless the task explicitly requires them.

## Workflow

Before writing code:

1. Investigate the relevant code paths.
2. Search for existing helpers, functions, types, services, hooks, components, and patterns.
3. Reproduce and confirm bugs or regressions.
4. Identify the root cause before implementing a fix.
5. Implement the smallest complete solution.
6. Run the relevant tests and checks.

Do not solve problems based only on symptoms or assumptions when the codebase can provide evidence.

Work autonomously within the requested scope. Do not request confirmation before creating focused, granular commits that follow these conventions.

Create commits as cohesive units of work become complete and verified. Do not accumulate unrelated changes into a single commit merely to reduce the number of commits.

Creating local commits does not require confirmation. Pushing branches, merging pull requests, publishing releases, or performing other remote or irreversible actions still requires explicit authorization unless the task already requests those actions.

## Test-driven development

Use TDD for new behavior, bug fixes, and regressions:

1. Write or update a test.
2. Confirm that it fails for the expected reason.
3. Implement the minimum required solution.
4. Refactor while keeping tests passing.

Bug fixes should include a regression test whenever practical.

Do not weaken, remove, or skip tests only to make a change pass.

## Reuse before creation

Research the codebase before adding new code.

Prefer, in order:

1. Reuse an existing implementation.
2. Extend an existing implementation.
3. Extract shared behavior.
4. Create a new abstraction only when necessary.

Avoid duplicating business rules, validations, formatting, error handling, styles, test setup, and integration logic.

New abstractions must have a clear responsibility and improve readability, testability, or reuse.

## File organization

Keep files below 400 lines of code whenever practical.

Files above 400 lines require a clear technical justification. Otherwise, split them into cohesive modules, helpers, components, services, types, or fixtures.

Do not reduce line count through compressed formatting or complex expressions.

A file should also be split when it has multiple unrelated responsibilities, even if it is below 400 lines.

## User interfaces

Before creating a component or hardcoding an interface:

* Search for existing components with the same or a similar purpose.
* Review the design system and shared style primitives.
* Reuse existing spacing, typography, colors, states, and interaction patterns.
* Prefer composition over duplication.

Keep interfaces and programming patterns consistent with the existing application.

Only introduce disruptive visual or architectural changes when explicitly requested.

## Code style

Use plain ASCII characters in code, comments, filenames, documentation, branch names, and commit messages.

Do not use:

* ANSI escape sequences.
* Emoji or decorative symbols.
* Smart quotes.
* Unicode dashes or arrows.
* Invisible or confusable Unicode characters.

Use simple expressions and regular keyboard characters.

Non-ASCII text is allowed only when required as product or domain data.

## Documentation

Documentation and comments must be understandable by anyone with repository access.

Only reference versioned repository artifacts, such as source files, tests, configuration, schemas, migrations, and architecture documents.

Do not reference conversations, harness messages, temporary plans, private reasoning, local notes, logs, or unversioned files.

Comments should explain non-obvious reasons, constraints, tradeoffs, or compatibility requirements. Do not comment code that is already self-explanatory.

## Scope and consistency

Keep changes focused on the requested outcome.

Do not introduce unrelated refactors, dependency upgrades, renames, formatting changes, or architectural replacements.

Follow existing conventions for naming, directory structure, testing, error handling, state management, data access, styling, and component composition.

When multiple patterns exist, investigate which one is current and canonical before adding another.

## Branch workflow

Never work directly on `main`.

Create a focused semantic branch for each change. Use lowercase names with hyphens:

```text
<type>/<short-description>
```

Recommended branch types:

```text
feat
fix
refactor
test
docs
build
ci
chore
perf
```

Examples:

```text
feat/customer-reminders
fix/inventory-negative-stock
refactor/shared-form-fields
test/session-expiration
```

Keep each branch focused on one feature, fix, or cohesive change.

Create granular local commits throughout the task without requesting confirmation for each commit.

Integrate branches into `main` using squash and merge. Do not use regular merge commits unless explicitly required.

The final squash commit must follow Conventional Commits and clearly describe the complete change.

## Conventional Commits

All commits must follow Conventional Commits:

```text
<type>(optional-scope): <description>
```

Common types:

```text
feat
fix
refactor
test
docs
build
ci
chore
perf
revert
```

Examples:

```text
feat(auth): add session expiration handling
fix(api): preserve validation error details
refactor(ui): reuse shared form component
test(inventory): cover negative stock regression
```

Descriptions must be concise, imperative, written in ASCII, and without a trailing period.

Keep commits focused and understandable. Each commit should represent one cohesive and reviewable step, such as a failing regression test, the corresponding implementation, or a focused refactor.

Do not request confirmation before creating a commit when its changes are within the requested scope and comply with these conventions.

Before squash and merge, ensure the pull request title can be used as a valid Conventional Commit message.

## Verification

Before completing a task:

* Run relevant tests.
* Run linting, formatting, type checking, and builds when available.
* Review the diff for duplicated code and unrelated changes.
* Confirm that new code follows existing patterns.
* Review file size and responsibilities.
* Confirm that documentation references only versioned artifacts.
* Confirm that the branch and final squash commit follow repository conventions.

Do not claim that a command passed unless it was actually executed. Clearly report any verification that could not be performed.
