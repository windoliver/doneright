# Rules for agents working in this repo

Read [docs/technical-design.md](docs/technical-design.md) before changing code. Keep this file under a page.

## The four invariants

No change may break these, in the core or in any extension:

1. An agent's word is never evidence. Only check results are.
2. Nothing missing or unrun passes. It's `BLOCKED` or `INVALID`.
3. Agents can't change checks or decisions.
4. A person sees only what needs a person, and never the same question twice.

## Proof

- Every fix starts with a test that fails for the reason you expect. Commit it before the fix, then make it pass without editing it.
- Test behavior, not implementation. A test that would still pass if every function it calls returned nothing is not a test. Rewrite it or delete it.
- Prefer end-to-end tests on real components for anything a user sees. Mock only at a real boundary you don't own.
- Never skip, retry until green, or delete a failing test to finish a task.
- A PR says `Closes #n` only when every acceptance criterion in the issue is proven. Otherwise it says `Part of #n` and lists what's open.

## Machine and safety

- Never kill processes by name or port. Stop only processes you started, by PID.
- Never write hooks into any repo's settings. This project installs hooks at user level only.
- No CI runs on push or pull request. Checks run locally, and everything runs once at release through a local preflight.
- No secrets or personal data in code, fixtures, logs or evidence. Redact.

## Scope

- Keep the core small: the record, gates, asks and adapters. A new core feature needs a reason no extension can meet.
- One PR per issue. Anything else you notice goes in the PR's notes as a follow-up.
