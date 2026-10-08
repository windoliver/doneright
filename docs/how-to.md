# How to use DoneRight

> [!NOTE]
> **Status: design.** This guide describes version 0, due November 8, 2026. Commands and file formats may change before then. Each section links the issue that builds it.

- [Install and remove](#install-and-remove)
- [What "done" means, drafted for you](#what-done-means-drafted-for-you)
- [Journeys](#journeys)
- [Prompts that work](#prompts-that-work)
- [The inbox](#the-inbox)
- [How your answer reaches the agent](#how-your-answer-reaches-the-agent)
- [Reading a verdict](#reading-a-verdict)
- [Require verdicts on GitHub](#require-verdicts-on-github)
- [Troubleshooting](#troubleshooting)
- [What's stored, and where](#whats-stored-and-where)

## Install and remove

Version 0 runs from a checkout of this repo and needs Node 22.19 or newer and Rust. The npm package, with prebuilt binaries, comes in R2 ([#41](https://github.com/windoliver/doneright/issues/41)).

```bash
git clone https://github.com/windoliver/doneright && cd doneright
npm install && npm run build
npm link     # puts `dr` on your PATH
dr setup
```

Run `dr setup` once per machine, from any folder. There's nothing to run per repo.

1. It reads your Claude Code and Codex history on your machine and writes what it learned into your wiki, `~/.doneright/wiki/`: the checks you keep asking for, how your stacks start, where credentials live (names only, never values) and the rules you've given agents. Every line names its source. It asks only where your history disagrees, at most five questions, each with the evidence on both sides and a recommended answer. A skipped one waits until an agent starts work it affects. A few one-time settings come with defaults already chosen.
2. It shows the exact hook entries it will add, at user level, for Claude Code and Codex. It merges with hooks you already have, and writes nothing until you say yes.
3. Nothing else. Each repo's checks are drafted later, from the wiki and the repo, as the next section describes.

Run it again any time to re-learn. It reads only what it hasn't seen. New facts go into the wiki, and a change to what decides "done" reaches you as a proposal.

- **Codex** asks you to approve new hooks once. Open Codex and approve them in `/hooks`.
- **Your existing hooks** keep working. `dr doctor` reports any other hook-based tool it finds and the order hooks run in.
- **Remove it** with `dr setup --undo`, which removes everything `dr setup` added. For a quick pause, `dr off` makes every gate watch-only, and `dr on` turns them back on.

## What "done" means, drafted for you

Each repo's checks live in `.doneright/done.yaml` ([#32](https://github.com/windoliver/doneright/issues/32)), but you don't write it. `dr setup` drafts it for each repo in your agent history, from what the repo already says ([#50](https://github.com/windoliver/doneright/issues/50)):

| Source | What it gives |
|---|---|
| CI workflows | The test, typecheck and lint commands your CI already runs |
| Package scripts, Makefile, justfile | Test commands, and how to start the app |
| `AGENTS.md` or `CLAUDE.md` | Rules such as "run lint before saying done" |
| Playwright or Cypress suites | The journeys you already have |
| Your past sessions in this repo | The checks you kept asking agents about |

Nothing runs in a repo you haven't trusted. Claude Code and Codex already record which folders you trust, and DoneRight reuses that, so setup asks nothing per repo. In each trusted repo, every candidate runs once in the background, in a clean environment, and one that runs zero tests, can't start or already fails is shown with its reason and left out. A repo neither agent trusts waits until you trust it.

Each draft reaches your inbox when an agent starts work in its repo, with its checks already run. You approve it once, keeping, dropping or editing each line. `dr` writes `.doneright/`, and you commit it like any other change. A repo you start after setup gets its draft the same way. Until you approve, verdicts there are watch-only. To see a repo's spec, draft one now, or change it in your own words, run `dr spec` in that repo.

**It keeps itself current.**

- A new CI step or a renamed script becomes one proposed change.
- When an agent changes a page that has no journey, the done gate asks it to propose one with `dr_propose`. The journey runs at once and its evidence is attached, but it decides nothing until you approve it.
- Agents can't write the spec. Each proposal reaches you once, and you can always edit the file by hand. It's reviewed like code.

A draft looks like this:

```yaml
claims:
  done:                                  # what "done" means for this repo
    checks: [unit, typecheck, journey:checkout]
  no-visible-change:                     # for refactors
    checks: [journey:all]                # each journey also runs on the merge base, and the two are diffed

checks:
  unit:
    type: command
    run: "npm test"                      # drafted from .github/workflows/ci.yml
    min_tests: 1                         # zero tests run is INVALID, never a pass
  typecheck:
    type: command
    run: "npm run typecheck"
  journey:checkout:
    type: journey
    file: .doneright/journeys/checkout.yaml
    accept: 10%                          # failure rate you accept; DoneRight works out the runs needed (29 here)

environment:
  up: "npm run dev"                      # or compose: docker-compose.yml
  ports: [web, api]                      # leased per worktree, so parallel agents never collide
  ready: { http: "http://localhost:${web}/health" }
  leases:
    test-account: { pool: 3, cooldown: 10m }   # one session at a time
```

What happens with it:

- **Checks run hermetically.** The environment is cleared except for allowlisted variables, the home directory is temporary, and each check gets its own process. A pass can't depend on your shell's state.
- **A fix must fail first.** For a fix, each check also runs on the merge base, in a temporary worktree DoneRight creates and removes. A check that passes there can't prove the fix, so it's `INVALID`.
- **Untrusted repos are watch-only.** A repo's `.doneright/` runs commands, so it runs only after you trust the project. Until then, DoneRight only watches.
- **The issue counts too.** If the agent is working on an issue, its acceptance criteria are added to the claim. A criterion without proof keeps the PR at "Part of #n" instead of "Closes #n".

## Journeys

A journey is a user task in plain words plus the end state to check ([#37](https://github.com/windoliver/doneright/issues/37)). It checks the result a user would see, not a label or a log line. Agents propose journeys, and you approve each one once. For an app that has none yet, ask your agent: "Propose DoneRight journeys for sign-up and checkout." A proposal looks like this:

```yaml
journey: checkout-with-test-card@1
as: a new visitor in a real browser
do: sign up, add one item, pay with the provider's test card
end state:
  - an order exists with status "paid" and the right total
  - the receipt email arrives within 60 s
  - no console errors and no failed requests
lane: local-journey
evidence: video, screenshots, network log
protects: class "a paid order isn't recorded"
```

- Agents propose the feature list in `.doneright/features/` the same way. DoneRight reports features that have no journey yet.
- When a change touches a file several features share, those features' journeys run too.
- Video, screenshots and the Playwright trace are kept as evidence, even after the environment is torn down.
- Credentials never go in journey files. They come from the leases in `done.yaml`.

## Prompts that work

DoneRight briefs every session when it starts, so agents already know how to claim done and how to ask you. These habits make it sharper.

**Set up a repo without writing config**

> Set up DoneRight for this repo, with journeys for sign-up and checkout.

The agent proposes the spec with `dr_propose`. DoneRight runs every check once on your stack, then sends you one approval with the journeys' videos.

**Fix a bug so it stays fixed**

> Fix #123. Reproduce it with a failing test first, commit the test, then make it pass and say done.

DoneRight checks that the test fails on the merge base and passes on your branch. A test that passes on both can't prove anything, so it's `INVALID`.

**Change code without changing what users see**

> Refactor the billing module with no visible change.

The journeys run on the merge base and on your branch, and screenshots, requests and timings are compared. If a button moves 8px, you see the two side by side.

**Build a feature that needs your eye**

> Add the new checkout layout. Show me the before and after when it's ready.

The screenshots come to your inbox as a taste call. Answer in `dr view`.

**Let agents work while you're away**

> Finish the open PRs on this branch. Ask me only taste calls and anything that costs money.

Slow checks run in the background. Verdicts post as commit statuses. Taste calls and approvals wait in one batch for when you're back.

**Stop the agent from asking twice**

Answer once, in `dr inbox` or `dr view`. The answer is saved as a decision, and the next time the same question comes up, it's answered for you and listed under "Decided for you". You can overrule it with one click.

**Things you no longer need to say**

- "Double-check your work" or "are you sure it's done?" The done gate does that, with evidence.
- "Send me screenshots." Journeys capture them, and `dr view` shows them.
- "Run the full suite before you push." That's what `done.yaml` is for.

**Optional snippet for `AGENTS.md` or `CLAUDE.md`**

```markdown
## DoneRight
- When you believe a task is done, call `dr_claim` or say "done". DoneRight runs the checks; don't claim done without them.
- If you need a decision from me, use `dr_ask` and keep working on whatever doesn't depend on it.
- Never edit files under `.doneright/`. Propose a change with `dr_propose` instead.
```

## The inbox

The inbox is the only thing DoneRight pushes to you ([#22](https://github.com/windoliver/doneright/issues/22)). An item reaches you for one of four reasons:

| Kind | Example |
|---|---|
| **Taste** | Does the new checkout layout look right? |
| **Approval** | Run the live payment test, about $0.40? |
| **Unblock** | The test account's sign-in expired; only you can renew it |
| **Exception** | A check you own can't fail, so it proves nothing. Fix it, or accept the gap? |

Everything else is handled without you. A block owned by someone else goes to that person as a drafted request. Asks are batched under a daily cap, and only an ask holding up a running session goes out right away.

```bash
dr inbox              # list what needs you
dr inbox answer 2     # answer from the terminal
dr view               # or answer in the local view, with screenshots side by side
```

## How your answer reaches the agent

| Agent | While it's working | When it stops with a question | When it's idle |
|---|---|---|---|
| Claude Code (terminal and desktop app) | At its next tool call | The stop is held until you answer, then the agent continues | A background waiter wakes the session when you answer |
| Codex | At its next tool call | The stop is held, and your answer becomes the next prompt | On your next message; Codex can't be woken from outside |

If an answer can't be delivered right away, `dr view` shows it as waiting.

## Reading a verdict

| Verdict | What to do |
|---|---|
| `PASS` | Nothing. The commit gets a green status. |
| `FAIL` | Nothing. The agent has the smallest failing case and keeps working. |
| `INCONCLUSIVE` | Nothing. More runs are scheduled; `dr show <id>` says how many. |
| `BLOCKED` | Only if you own the cause. It's in your inbox with the exact gap. |
| `INVALID` | Fix the check, or ask an agent to propose a fix. Agents can't change checks on their own. |

A verdict covers one commit. A new commit marks it stale, and only checks whose inputs changed run again.

## Require verdicts on GitHub

Each verdict posts a commit status named `doneright/verdict` ([#46](https://github.com/windoliver/doneright/issues/46)). Only `PASS` is green; `INCONCLUSIVE` and `BLOCKED` stay pending. To make it required, add `doneright/verdict` to your branch protection or ruleset in GitHub's settings. DoneRight never changes branch protection itself. PR comments and `dr report` come in R2 ([#49](https://github.com/windoliver/doneright/issues/49)).

## Troubleshooting

| Symptom | What to do |
|---|---|
| Something seems off | Run `dr doctor`. It checks hooks, ports, disk, sign-ins and token lifetimes, and names the fix. |
| DoneRight is in the way | Run `dr off`. Every gate becomes watch-only at once; `dr on` restores them. |
| The hub isn't running | Agents keep working: hooks let actions through when the hub is down. The default safety rules still block, such as killing processes by name. |
| A check you expected isn't in the draft | `dr spec` lists what it found and why it left anything out. Say what to add in your own words, or ask your agent to propose it. |
| A check is `INVALID` with zero tests | The command ran but no tests executed. Check its filter or path. |
| A check is `BLOCKED` | `dr show <id>` names the missing piece and its owner. |
| You want to report a bug | Run `dr debug bundle`. It writes a redacted bundle of logs, versions and doctor output to attach to an issue. |

## What's stored, and where

| What | Where | Notes |
|---|---|---|
| The record | `~/.doneright/ledger.db` | An append-only SQLite event log |
| Evidence | `~/.doneright/evidence/` | Screenshots, video and traces, named by content hash; kept 30 days unless a decision cites them |
| Transcript archive | `~/.doneright/archive/` | Copies of agent transcripts, saved before the agents' own cleanup deletes them |
| Your wiki | `~/.doneright/wiki/` | Markdown pages of what DoneRight learned, every line with its source. Edit them freely; your edits are kept. Agents read them and propose changes, but never write them |
| Repo specs | `.doneright/` in each repo | Drafted by `dr setup`, approved by you and reviewed like code; agents can't edit them |

Keys and tokens are redacted on the way in. Nothing leaves your machine unless you install and turn on an extension that syncs.
