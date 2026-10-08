# Measure and prove every change

> Writing code is cheap now. Knowing a change is right, and what it cost, is the bottleneck. DoneRight measures every call, feature and change in money, speed, your time and quality. It proves each change held before you have to ask, with evidence the tool makes itself, honest verdicts, and receipts tied to the exact build that ships. Agents do the work and people make the taste calls. It starts as an open-source tool for small teams that you run and view on your own machine, beginning with proof of "done". Infra cost for the vendors you use arrives early. Full cross-vendor FinOps and a hosted team version come later, if it earns them. An enterprise profile covers large organizations. The core is small: a record, gates, asks and agent adapters. Everything else ships as extensions you can turn off, replace or write yourself.

**Agents do the work.** **The tool owns the evidence.** **People make the calls.**

How version 0 is built is in the [technical design](./technical-design.md). The repo, with the build plan as issues, is [windoliver/doneright](https://github.com/windoliver/doneright).

## What teams building with agents keep paying for

- **"Is it really done?"** In a 30-day pilot, a founder asked about 600 times, and a quarter of all messages were nudges or proof requests. METR found at least 16% of "successful" long agent runs weren't.

- **"Bugs I fixed come back."** The pilot found 73 regressions that returned at least once. About half got past tests that checked a stand-in, such as a mock, a label or a small fixture.

- **"The check said pass, but it didn't really run."** In the pilot, a weekly full test suite stayed red for three weeks without anyone noticing. Proof run on stubs or in a different environment still reads as green.

- **"I can't see what we spend, and the numbers disagree."** The pilot's two apps were billed by 25 vendors. For two days, the app's own usage table showed $3.61 while the shared API key behind it spent $116.

- **"It got slower and nobody noticed."** Speed limits like "under 1 second" lived only in chat. The budgets that did exist kept getting raised: prompt-size limits went up 649 times and down 37.

- **"My teammate and I overwrite each other's work."** Agents resolve conflicts by keeping their own side. In April, GitHub's own merge queue silently reverted earlier changes in 2,092 pull requests.

- **"It passed on fakes, then failed for real."** In the pilot, a booking flow passed 42 of 42 checks against fake widgets and failed on the real sites. Of the 73 returning regressions, 38 involved an environment difference and 22 a fixture that hid the bug.

- **"It's fixed on main, but users still have the bug."** OpenClaw's 2026.9.8 release shipped two days after a Windows fix was merged, without it. In hermes-agent, "fixed on main" appears in 275 issues.

## A small core, and everything else as an extension

From first principles the product has one job: turn an agent's claim into a verdict without a person, and bring a person only the calls that need one. What that job needs is the core. Everything else, including most of this document, is an extension you can turn off, replace or write yourself. This follows [pi](https://github.com/earendil-works/pi), a minimal agent harness with strong defaults. It leaves features such as sub-agents and plan mode to packages, and builds every interface on one session record.

### The core has four parts

- **Record:** an append-only event log plus the evidence folder. Every view, agent and extension reads it.
- **Gates:** when a trigger fires, a gate runs the checks for a claim and returns one of five verdicts. The done gate, a guard before a command and a check on a PR are all gates with different triggers.
- **Asks:** the only thing a person sees. Resolvers try to answer each ask first. What's left goes to the inbox, and the answer is saved as a decision and delivered back to the session.
- **Adapters:** one contract for the hook points every agent has (session start, before and after a tool, prompt submit, and stop), plus delivering your answer and starting a session. The fast rules live in the hook client, so they hold even when the hub is down.

### Six concepts and four invariants

**Event, claim, check, verdict, ask and decision.** The other terms in this document are fields or extensions. A reading is an event with a number, a guard is a gate in block mode, a receipt is an exported verdict, and an owner is a field on an ask.

- An agent's word is never evidence; only check results are.
- Nothing missing or unrun passes. It's BLOCKED or INVALID.
- Agents can't change checks or decisions.
- A person sees only what needs a person, and never the same question twice.

No extension can break these. The other rules in the framework are defaults of the extensions that need them.

**Only where you're needed.** An item reaches you for one of four reasons. Everything else is handled without you. Nothing else is pushed, but the whole record stays visible in the DoneRight app whenever you look.

| Reaches you | Never reaches you |
|---|---|
| **Taste:** a judgment only you can make, such as whether a screen looks right or a trade-off is acceptable | **PASS:** recorded, with nothing to do |
| **Authority:** anything irreversible, outward-facing, or above a budget you set | **FAIL:** goes back to the agent with the smallest failing case |
| **Unblock:** something only you can provide, such as a sign-in, access, a key or an unlocked device | **INCONCLUSIVE:** more runs are scheduled within the budget |
| **Exception:** something the system can't settle, such as a premise review after two failed fixes or two rules that conflict | **A question you've answered before:** a resolver answers it from your saved decisions. A BLOCKED item owned by someone else goes to that person with the request drafted. |

Asks are batched and interruptions have a daily cap. Only an ask that is holding up a running session goes out right away. The product's main number is how many asks reach you per merged change, which should fall while verdicts hold.

**Nothing to do after install.** Installing the app or the CLI is the only step: there is no setup screen and no Settings page. On first launch, or the first `dr` command, it sets itself up. It reads your Claude Code and Codex history and their memory files, on your machine, and writes what it learned into a local wiki: what you keep asking agents to check, your taste calls, how your stacks start, where credentials live (names and paths, never values), and the rules you've given agents. The wiki is plain markdown you can read and edit, and every line names its source. Most of what it finds is clear, so it's just recorded, and nothing is asked at install. Where your history disagrees, or where one remark would become a standing rule, it keeps at most five open questions, ranked by what a wrong guess would cost, and asks each one when it matters, as a normal ask with the evidence on both sides and a recommended answer. Which way `api` starts is asked the first time an agent starts `api`, and a skipped question waits until it matters again. New evidence never overwrites an answer you gave; it asks again. A few one-time settings come with defaults already chosen: which agent does its reading, which accounts belong to which repos, how much of each plan window to keep for you. Then it starts the hub and adds its hooks to Claude Code and Codex in watch-only mode, so no verdict blocks and no stop is held until you approve something, such as a repo's drafted checks when an agent first starts there. A few safety rules apply from the start, such as no killing processes by name or port. Codex still asks you once to trust the hooks; that prompt is Codex's own. It tells you in one line what it added and how to remove it. None of this touches a repo. Each repo's checks are drafted later from the wiki and the repo, run once in the background in repos you already trust in Claude Code or Codex, and come to you when an agent starts work there; until you approve them, verdicts there are watch-only. `dr spec` shows or changes one repo's spec. After that, the wiki grows from the hooks as sessions happen. `dr setup` still exists for scripts and CI, where it does the same steps without asking anything, but nobody has to run it. DoneRight brings no model of its own: when it needs to read free text, it runs the agent you already use, headless, on your own sign-in, with your own skills and settings.

**Change anything by saying it.** To change anything, say it in words, in the app's box or to your agent: "stop checking lint in shop", "forget my screenshot habit", "remove DoneRight from Codex", or "learn again" to re-read your history. Each comes back as a proposed change you approve, like every other change, and your own agent does the work, on your plan. Removing DoneRight from an agent restores that agent's settings exactly. The same box is where you add what you need (Add anything, R2).

**Your words and your screenshots.** Buttons are a shortcut, never the only answer. Say what you want instead: words that change a spec, a playbook or a check come back as a diff, and nothing is written until you apply it. Words or a screenshot answering a taste call go to the agent as your answer and are saved with the decision. A screenshot you paste into a session is kept too, by hash, with the message it came with and the change it's about. If it shows a bug, the fix has to pass a check made from it, either the end state it shows or a visual diff. If it's a taste remark, it's saved as a decision with the screenshot as its baseline, so agents see it before they touch that screen again and you never send it twice.

### The extension interface

```
// an extension is a module the core loads, the way pi loads its extensions
export default function (dr) {
  dr.on("agent.stop", checkTheClaim)            // also session.start, tool.before, tool.after,
                                                // pr.opened, merged, released, schedule
  dr.registerCheck("journey", runJourney)       // command, journey, property, eval …
  dr.registerResolver(answerFromDecisions)      // tries an ask before a person sees it
  dr.registerSource("transcripts", readSessions)  // turns outside data into events
  dr.registerEnvironment("stack", bringUpStack) // what checks need, with leases
  dr.registerView("side-by-side", renderShots)  // panels, PR comments, reports, exports
  dr.registerAdapter("codex", codexHooks)       // hook points, answer delivery, starting a session
  dr.on("tool.before", headsUp)                 // a handler may add context for the agent, or deny with a reason
  dr.startSession({ agent, worktree, prompt })  // through the agent's adapter, on your own sign-in
  dr.registerScheduler(pickAndPace)             // which agent, plan and model takes a task, and when
}
```

### First-party extensions

- **Agent adapters:** Claude Code and Codex first, then pi and Cursor.
- **Prove "done":** the done gate, journeys, environments and spec drafting, with their views in the app, on by default.
- **Inbox and decisions:** batching, resolvers and the keep-or-remove list.
- **Parallel sessions:** the session map, heads-ups, dry merges and leases.
- **Plan usage and readings:** plan windows, tokens per task, speed, time, quality and spend.
- **Reports:** GitHub statuses, the PR comment, `dr report`, receipts and exports.
- **Playbooks:** `dr create`, the scheduler and picker, scores and replayed improvements.
- **Money and infra**, **Keep** (guards, memory, fix reach), **CI**, the **team server** and the **enterprise preset**, later.

### Taken from pi

- Strong defaults and a small core. A feature that isn't the core's one job ships as a package.
- One record behind every interface. The terminal, the app, MCP tools and hooks all call the same core.
- Packages come from npm or git, pinned to a version. They install at user level, and a project's packages load only after you trust the project, because extensions run as code.
- The extension API has one test of whether it's simple enough. Given only its docs, Claude Code and Codex each build three extensions from one sentence each, on the first try: a line for your screenshots, a helper that points at things on your screen, and an email monitor.
- Open formats ship as contracts with conformance tests, the way pi publishes its telemetry contracts, so other tools can prove they're compatible.
- Outside PRs to the core are closed automatically and reviewed in a daily batch, as pi does. Outside work goes into extensions.

### Every use case, on the same four parts

| Use case | What you see | Extension | Core parts it uses | Lands |
|---|---|---|---|---|
| "Is it really done?" | A verdict with its proof: journeys, screenshots, the merge base | Prove "done" | Gates, the record, adapters (holding the stop) | R1 |
| Agents know how you work | Your wiki: what you check, how stacks start, your rules and answers, each with its source | Wiki | The record (sources, decisions), adapters (session-start context) | R1 |
| Which part is moving, and what's proven | The project map: each part with its sessions, issues and PRs, evidence and gaps | The app (Map) | The record, gates | R1 |
| Only what needs you | One count, asks in batches, questions decided for you | Inbox and decisions | Asks, the record | R1 |
| Your answer reaches the agent | The answer inside the running or idle session | Agent adapters | Adapters (delivery), asks | R1 |
| Many agents at once | Heads-ups, hard stops, leased ports and accounts | Parallel sessions | The record (the session map), adapters (context and deny), gates | R1 rules, R2 |
| Plan limits and spend | Each plan's windows, tokens per task, spend | Plan usage and readings | The record (sources) | R2 |
| Proof others can check | A GitHub status, the PR comment, the report, a receipt | Reports | The record, gates | R1 to R3 |
| Start an app | A playbook, lanes, scheduling within your plans, the release | Playbooks | Adapters (starting a session), gates, asks, the record | R6 |
| Add what you need | The app's box: one sentence becomes a panel, a watcher or a shortcut, built by your own agent and installed after one approval | Add anything | Adapters (starting a session), gates, asks, the record | R2 |
| Pick between parts by evidence | Try a second memory, check runner or scheduler next to the one in use, as a copy, then compare them on a replay of your own sessions | Trying side by side | The record, the extension host, asks | R2 |
| See how any piece of work moves | The Flow view: a playbook's steps as one map, who does each one (the agent, a check or you), and where each task is now; claims and builds use the same drawing at small scale | Flow view | The record, gates, asks | R6 |
| Money and infra | Bills, findings by dollars, the infra map | Money and infra | The record (sources), gates (cost guards), asks | R2, R3 |
| Keep it fixed | Guards for whole classes of bugs, memory, fix reach | Keep | Gates (block mode), asks, the record | R4 |
| CI and teams | CI only when ready, a shared inbox | CI, the team server | Gates, asks, the record | R5, later |

Every use case is an extension on the same four parts, so the core stays small. Parallel sessions and playbooks needed two more extension points, not a fifth part: a hook handler can add context for the agent or deny with a reason, and an extension can start a session through an adapter and register the scheduler that picks the agent, plan and model.

**What we removed.** Fourteen concepts became six, and ten rules became four core invariants plus extension defaults. Profiles became presets, which are lists of extensions with settings. Pushed dashboards and alerts are gone, while the app still shows the whole record when you look. The enterprise track is a preset plus adapters, not a separate product. Each agent's delivery routes moved into its adapter.

## Measure, prove, keep, and let people decide

### Measure

Four readings for every call, feature and change:

- **Money:** agent and model spend and cost per successful task first. Bills for the vendors you use arrive in R2 and R3, and every other vendor follows later. Where cost isn't dollars, it's GPU-hours and quota.
- **Speed:** broken down by stage and by turns per task.
- **Your time:** rework, review wait, and blocked time by cause.
- **Quality:** evals, errors and retries.

### Prove

Each change gets a verdict from evidence the tool makes itself. The check fails before the fix and passes after, every reading holds within its margin, and a receipt is tied to the exact build. Agents can't edit the checks.

### Keep

Every fixed failure leaves a guard for its whole class, not just the one bug. Each guard is measured by how often it fires and how often it's wrong. It moves from watching to warning to blocking, and is re-proven whenever a model, tool or price changes.

### Decide

People get only the taste calls, the open questions and the approvals, batched and side by side. Each answer is saved as a baseline or a rule, so it's never asked twice.

Prove and Decide are the core. Measure and Keep are first-party extensions, on by default. The concepts, rules and open formats behind these four parts are in the [framework](#the-core-everything-is-built-from) below.

| How it works with Claude Code, Codex, pi and Cursor | What happens |
|---|---|
| Reads their transcripts | Time, rework, corrections, "is it done?" asks, each agent's "still open" list, token cost |
| Hook: agent tries to finish | The done gate runs. If proof is missing, the agent is told exactly what's missing and keeps working. |
| Hook: before each command | Guards check it, for example blocking kills by name or unapproved paid runs |
| Hook: session start | The agent receives open items, guards and environment status |
| MCP tools | Start the environment, run the proof, add an open item, ask a person a taste call, check a guard |
| Live channel | Every session registers with the local hub under its session ID and worktree. An agent's question appears in your inbox the moment it's asked. Each answer shows whether it's still waiting or was delivered, and by which route. The table below shows how fast it gets back to each agent. |

**How your answer gets back to the session.** No agent has a timer event for hooks, so a waiter starts when the turn ends. The Stop hook starts one every time, and the agent can also start one with `dr wait` when it thinks it's done. The waiter holds one connection to the hub and returns the moment you answer.

| Agent | Working | Stops with a question for you | Idle | Ended |
|---|---|---|---|---|
| Claude Code | At its next tool call (PostToolUse hook) | The Stop hook holds the turn until you answer, then the agent continues with your answer | A background waiter wakes the session when your answer arrives. The Stop hook arms one at every stop (`asyncRewake`), and a background `dr wait` the agent started works the same way | Next session in that worktree (SessionStart hook) |
| Codex | At its next tool call (PostToolUse hook) | The Stop hook holds the turn (600 s by default, can be raised), then your answer becomes the next prompt | Neither a background hook nor a finished background command can start a turn, so on your next message (UserPromptSubmit hook). In sessions `dr` starts through app-server: right away | Next session (SessionStart hook) |
| Cursor | At its next tool call (postToolUse hook) | The stop hook holds, then sends your answer as a follow-up message (5 per chat by default, can be raised) | Local chats: on your next message. Cloud agents: right away, through the Cloud Agents API | Next session (sessionStart hook) |
| Any other agent, local or in the cloud | Without an adapter, nothing reaches the session: no hold and no delivery. Its PR is still checked: DoneRight runs the done gate on the PR's commit in a temporary worktree and posts the verdict. An adapter, in any language over JSON-RPC, adds the rest when the agent has hooks. | — |  |  |
| pi | Right away, steered in after its current tool calls | Right away, queued for when it finishes | Right away: the extension starts a new turn (`pi.sendUserMessage`) | Next session (`session_start` event) |

The hook-started waiter is the backstop for when the agent forgets. Holding costs nothing, but the session looks busy until you answer or the hold times out, so `dr` holds only when the agent left a question for you. A question in the middle of a task works the same way: the agent calls `dr ask` and waits on the call. Each waiter exits with its session and claims an answer only once, and `dr` confirms delivery from the transcript. Claude Code can also wake on a timer (session crons), but every tick is a full model turn, so `dr` doesn't use them. In Codex, a wait the agent runs in the foreground can turn into repeated polling that burns tokens. Nothing here needs a startup flag. Claude Code's channels do, and they don't run in the desktop app. Answers enter the agent as instructions, so the hub accepts them only from your terminal and the app.

Tested October 4, 2026 on Claude Code 2.1.289, headless: a held stop continued the agent with the answer. An idle session woke 1.4 seconds after the answer reached the hook's waiter, and 2.5 seconds after it reached a background waiter the agent had started. Hooks also run in the Claude desktop app: a desktop session ran its SessionStart and WorktreeCreate hooks on October 6. Not yet tested: a held stop there, Codex, the Codex app and Cursor. Two open reports say hooks didn't run in the Codex app ([openai/codex#33992](https://github.com/openai/codex/issues/33992), [#47607](https://github.com/openai/codex/issues/47607)), so the Codex app is tested before anyone relies on it. Sources: Claude Code [hooks](https://code.claude.com/docs/en/hooks) (background hooks, `asyncRewake`) and [channels](https://code.claude.com/docs/en/channels); Codex [hooks](https://developers.openai.com/codex/hooks) (background hooks don't start a turn) and [app-server](https://developers.openai.com/codex/app-server); [openai/codex#47193](https://github.com/openai/codex/issues/47193), [#32188](https://github.com/openai/codex/issues/32188) (background completion doesn't wake) and [#38495](https://github.com/openai/codex/issues/38495) (polling cost); Cursor [hooks](https://cursor.com/docs/hooks) (`followup_message`, `loop_limit`); pi [sendUserMessage example](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/examples/extensions/send-user-message.ts).

**Memory: one wiki for every agent, built from theirs.** Each agent already keeps memory, and each keeps it alone. Claude Code writes notes per repository and loads the first 200 lines of their index. Codex summarizes past sessions and loads only 2,500 tokens of that summary. pi and Cursor read instruction files only. None sees the others' notes, and none enforces them. `dr` reads all of them, plus the transcripts, and compiles one local wiki: markdown pages, every line naming its source, that every agent reads when it starts and searches through MCP. Agents never write it; they propose, and your answers and edits are kept as decisions. Because it's rebuilt from its sources, it can't drift on its own. From the wiki, each lesson becomes one of three things:

- **A guard**, when the lesson can be checked, such as "never kill processes by name".
- **A rule**, written once into every agent's instruction file, so all agents read the same text.
- **A removal**, when it's stale, unused or contradicted, applied to every agent's memory.

A repeated mistake is fixed at the highest level that works. The order is the code's structure first, so the wrong path doesn't exist, then types, then a lint or CI check whose error names the fix, then a behavior test, then a hook. Text in an instruction file comes last and only for judgment calls, because nothing fails when an agent skips it. Each new check is proven on a real past mistake from the ledger. A table pairs every rule with what enforces it. A rule that's broken again with nothing enforcing it moves up the ladder, and an exception sits on the offending line with a reason, an expiry date and a person's approval.

You don't read memory files. The inbox shows a short ranked list, and you answer keep or remove. Lessons that keep recurring or keep being broken come first, weighted by the hours or dollars they cost. Next come memories that disagree between agents, then facts about code that no longer exists. Each answer is saved as a decision, so an item never comes back, and unused items expire on their own.

**Policy: agents stop asking what you've already answered.** Agents stop to confirm even when every permission is granted, because their own instructions tell them to. In the pilot's last 30 days, agents stopped with a question 681 times, 94% of them with every permission already granted. About one reply in five was a standing answer: a plain "yes", "your call", "fix all of them" or "create PR". `dr` checks policy wherever an agent can stop to ask:

- **It ends its turn with a question.** The Stop hook reads the question.
- **It calls its ask tool.** A hook can answer it, which Claude Code supports for its question tool.
- **It hits a permission prompt** its mode doesn't cover. The permission hook allows or denies it.

Policy is your instruction files, your saved decisions and your guards. When it covers the question, the hook answers and the agent goes on. When a rule forbids it, the hook names the rule and what to do instead. Taste calls, spending, anything irreversible or outward-facing, and first-time questions still come to you. An answer you keep giving the same way is proposed as policy in the keep-or-remove list, for example "open the PR once the done gate passes". Policy can never allow more than the permission mode and guards do, and every answer it gives is logged.

**Repeated errors fix themselves.** Every failed tool call is recorded by its error signature, like blocked time. In the pilot, 149 signatures came back in three or more sessions and made up 53% of 3,613 failed calls. Most had a known fix that each new session had to rediscover: commands too complex for the worktree check, waiting with `sleep` where the harness wants a watcher, macOS `sed` and zsh glob syntax, and `cd` into a folder that isn't there. When a signature comes back, `dr` acts in this order:

- **A known fix, before the error.** A hook rewrites or stops the command with the fix, or the session starts already knowing it.
- **A fix at the root**, as a change with proof: the check fails before and passes after, and a guard keeps it fixed.
- **A BLOCKED item** for its owner when only someone else can fix it, with the request drafted.

Each fix is measured like a guard: how often it fired, whether the error stopped, and whether it was wrong. It moves from suggesting to applying with a note to applying quietly, with your OK at each step. A fix never kills processes by name, deletes data or buys anything.

**Two rules hold it together.** A change may improve one reading only if the others hold. And nothing acts on a reading it hasn't checked: before any cap, rollback or optimizer fires, the number must be fresh, on the right cost basis, and agree with a second source.

| Verdict | Means | Example |
|---|---|---|
| `PASS` | Proven: the check failed before the fix and passed after, and the readings held within their margins over enough runs | 200 replayed tasks, slowest runs down from 22 s to 8 s, success up from 94% to 95% |
| `FAIL` | A check or a reading got worse; the agent gets the smallest failing case back | The page count no longer matches the email count when two writes land during a send |
| `INCONCLUSIVE` | Not enough runs to tell a real change from noise | 20 runs can't show a 2-point quality change; the tool says how many would |
| `BLOCKED` | Something outside the code is missing: an environment, an access grant or an approval. It has a cause, an owner and an age. | The test account's sign-in expired two days ago; owner: you |
| `INVALID` | The check itself is broken or can't fail | A test that copies its expected value from the code it tests |

## A browser task got slow

1. **Measure.** A browser agent's "read this page" task now takes 22 seconds in its slowest 5% of runs, up from 9 seconds before last Tuesday's deploy. Breaking it down by stage shows 61% is one model call reading a full-page screenshot. speed 22 s ↑ $0.031 / task 94% success
2. **Change.** An agent sends page text instead of the screenshot and uses a faster model for reading.
3. **Prove.** The tool replays 200 recorded tasks before and after, in an environment that matches production. `PASS` speed 8 s ↓ $0.019 / task ↓ 95% success, held The receipt is tied to the build that ships, and any CI or deploy step can verify it.
4. **Keep.** The task gets a 10-second speed budget that can only tighten, plus a guard for its class: any page-reading step that sends full screenshots.
5. **Re-check.** The meter checks again after 7 and 28 days, and again whenever the model or browser library changes.

The numbers are illustrative.

## Prove it the way a user would use it

Agents are weak test engineers, and rules in an instruction file don't fix that at scale. Asked to write tests, they write unit tests that check their own code against mocks. So the tool owns the three parts agents get wrong: the environment, the test plan, and whether a test proves anything. Of the pilot's 73 returning regressions, 38 involved an environment difference, 34 a test that ran late or only locally, 27 a test of a mock or an implementation detail, and 22 a fixture that hid the bug. In the last two weeks, the founder asked for checks on real surfaces 176 times, asked for screenshots 158 times, and asked "done and validated?" 505 times.

| Piece | What it does |
|---|---|
| Journeys | A journey is a user task in plain words plus the end state to check. An agent drives it the way a user would, in a browser, a desktop app, a phone call, a text message or an API, on a real stack. It checks the end state, never a label or a receipt, and keeps video, screenshots and recordings in the ledger. Once a journey passes, it's saved as a script that replays cheaply. The agent drives it again only when the script breaks, and it can't change the end-state check. |
| Feature map | Journeys start from a map of the app's user-facing features, built from its routes, commands and menus. A periodic pass reads each feature's code and drives every feature live. Drift in the map gets fixed and a gap in the driver gets fixed, but a feature that no longer works is reported as a regression, never written out of the map. Features with no journey are a reading. |
| A test plan the tool checks | Before writing tests, the agent writes a five-line plan: journeys, end state, environment, what isn't covered, and cost. `dr` checks it: real services on the path being claimed, end-state checks, a test that can fail before the fix, and a cost within budget. You see the plan only when it involves taste or money. |
| Environment manifest | One file per repo lists the services, production-shaped seed data, health checks, teardown and cost, and which external services are real, a provider's sandbox or recorded in each lane. Test accounts, phone numbers, simulators and screens are leased to one session at a time, like ports. Each real resource has a cap, a cooldown and a cost estimate before every run. The proof records the environment it ran in, and a pass anywhere else doesn't count. |
| Stand-in detection | A test that still passes with its real dependency switched off is checking a stand-in. It can't prove anything a user would notice. |
| The claim sets the lane | Static checks, then unit, integration on real parts, a local journey, a live journey with real accounts (paid, budgeted and run only when asked), and production monitoring. "No visible change" needs a journey diff against the merge base. "Users can pay" needs a live journey. Results are cached per build and environment, so the costly lane runs once. |
| Tests must bite | The cheapest check comes first: a test that still passes when every function it calls returns nothing is INVALID. Then a new test must fail on the old code, catch a fault planted in the changed lines, and name the journey or failure class it protects. A test is deleted only with a receipt showing every journey is still covered. |
| Agents use the app | On a schedule, an agent uses the app on the real stack as a user would and files what it finds, with video, to the inbox. Each confirmed bug becomes a journey. Journeys are also drawn from real usage, such as session replays. |
| Flaky means INCONCLUSIVE | Unpredictable journeys run several times, and a flaky result is never rerun until it's green. A quarantined test has an owner and an expiry date, never a silent skip. |

```
journey: checkout-with-test-card@1
as: a new visitor in a real browser
do: sign up, add one item, pay with the provider's test card
end state:
  - an order exists with status "paid" and the right total
  - the receipt email arrives within 60 s
  - no console errors and no failed requests
environment: { db: seeded, payments: provider-test-mode, email: real-test-inbox }
lane: local-journey   runs: 3   budget: $0.05
evidence: video, screenshots, network log
protects: class "a paid order isn't recorded"
```

Testing guidance arrives when it's needed, not as instruction-file text. When the agent tries to finish, the done gate names the journey that hasn't run on the real stack, and a testing skill loads only when the plan calls for one. Playwright, Maestro and the agents' own browser and computer-use tools drive journeys. Qlty CLI can run static checks, but it's source-available under the Business Source License 1.1, so it stays optional.

## Terminal first, one native app for screenshots, hosted later

The ledger is for agents, but people need a place to look. Start with no hosting at all. Quick numbers and follow-ups show in the terminal. Screenshots, side-by-side taste calls and charts open in the app, one native desktop app on your own machine. It's native on macOS first, and the same app runs on Windows and Linux later. There's no web view: HTML is kept only for reports you share. A team server comes only when a team needs a shared view across people, and teammates use the same app pointed at it.

### Terminal

```
$ dr status
this week   rework 6.5 h ↓   "is it done?" 41 ↓   blocked 3.2 h (sign-in)
verdicts    PASS 37  FAIL 4  INCONCLUSIVE 2  BLOCKED 1
waiting on you: 3 taste calls · 2 open items · 1 guard promotion

$ dr inbox
1  taste    checkout button moved 8px left        → dr open 1
2  open     payment email never tested live        needs a run · $0.40
3  open     async mode untested                    gap · accept or file
4  guard    no-full-screenshot-reads: warn → block needs your OK
```

### The app (`dr open`)

The mocks and numbers are illustrative. The same evidence also appears in the PR comment (verdict plus screenshots), the status line and the Monday brief.

**What the app shows.** You open it with three questions: does anything need me, is the work done, and what did it cost. So it has three views, in that order. Only the first ever pushes, and nothing is taken away by keeping the rest quiet: every detail is one click deeper.

- **Needs you:** each ask as one question, the one piece of evidence that answers it, and two buttons, or your own words or a screenshot. Below it, what's coming in your next batch, and everything decided for you, each with a one-click overrule.
- **Work:** every session and change on one line, led by a plain sentence such as "Not done: the receipt shows $18.00". Open one for its verdict and its end-to-end run: how many runs came back clean, a screenshot of each step, and the end state with a highlight of what was checked beside it. Failing and running checks come next; passed checks fold into one line. The video, trace and receipt are one click deeper, and `dr report` puts all of it in one HTML file. The repo's spec is here too.
- **Numbers:** readings outside their band, or that changed, come first; the rest fold into one line. Money, the infra map and the guards that fired are sections here, and extensions add their own.

One count, the things that need you, reads the same everywhere: the menu bar, the status line and the first line of `dr status`. Color means status and nothing else: the five verdicts and "needs you". Highlights never cover a screenshot: what each check looked at is marked in a strip beside it.

**Flow.** With playbooks (R6), the app adds a Flow view: a playbook's steps drawn as one map, like a transit map, with each task moving along it. Each station shows who does that step: the agent, a check, or you. Lines light up as work moves: running, sent back (with its round, such as 1 of 3), passed, or waiting on you. Click a station to see its gate and evidence. The pipeline map on each claim and build, in the Work view, is the same drawing at small scale. It isn't only for code. Any agent work fits the same model: the agent works, a check proves the result, and you're asked only for taste, approval or what only you can do.

- **Fix a bug:** issue → agent → checks → you (taste) → PR.
- **A weekly report:** pull the numbers → draft → numbers checked against their sources → you (tone) → send. Sending is outward-facing, so it needs your approval.
- **Book travel:** options → policy and budget check → you pick → the agent books, with your approval before it pays → the receipt is checked.

DoneRight's adapters today are for coding agents: Claude Code, Codex, pi and Cursor. The core (claims, checks and asks) and playbooks already work for the non-code tasks those agents do.

**Where it runs.** Every surface reads one data model: the ledger in a local SQLite file and the evidence folder beside it. Our own views show only what no other tool shows: the inbox, verdicts with their evidence, and receipts. Long-term charts go to the tools you already use.

| Surface | How it's built and hosted | Who sees it |
|---|---|---|
| Terminal | `dr status`, `dr inbox` and `dr show`, from the same CLI. Agents read the same data over MCP. | You and your agents |
| The app | One native desktop app, built with gpui-kit. `dr open` opens it at what needs you, or at a session, change or report, such as `dr open #128`. The hub serves it and the CLI over a local socket, with a new token each launch, because answers given there reach agents as instructions. The hub refuses any request carrying a browser Origin, so no web page can reach it. There's no account and no Docker. Away from your Mac, asks still reach you as notifications that carry the answers. If a browser view is ever needed, gpui-kit's WebAssembly target is the option. | You |
| Report file | `dr report` writes one self-contained HTML file for a change, with the verdict, screenshots, video and receipt. It opens offline in any browser or in the app's webview, and attaches to a PR or a CI run, the way Playwright's test report does. | Anyone you send it to |
| Where you already look | A PR comment and check run, the agent's status line, the Monday brief, and readings exported as OpenTelemetry metrics so Grafana or PostHog can chart trends. | Your team |
| Team server, later | Teammates use the same app, pointed at the team server. Each machine pushes its ledger events, content-free by default, with evidence only for projects that opt in. It ships as one Docker image with Postgres and S3-compatible storage behind your sign-in, or as Workers, D1 and R2 behind Cloudflare Access for a small team. Enterprises run it in their own cloud. | Your team, after Decision 2 |

## FinOps built on the same loop, run by agents

In the pilot, a founder raised cost or infra 154 times in five weeks, across 48 sessions, and agents spent about 70 hours on it. A third of those asks were not "cut this". They were "is this number right?", "what will it cost?" and "did it hold?", which are validation questions. Fixes also kept coming undone because each was one-off:

- A stopped staging service was running again two days later.
- A shared key was approved for reuse the day after an audit flagged it.
- Backups grew to 203 GB for an 18 GB volume.

Incumbent FinOps tools see, by our count, about 7 of the 25 vendors that billed the pilot. Their fixes mostly act on AWS and Kubernetes, and none checks quality. Here every cost job runs through the same framework: checked readings, a finding, a change with proof, a re-measure, and a guard.

| Cost job | Finding | Change with proof: "held" means | Guard | Check before acting |
|---|---|---|---|---|
| Why is the bill high? | The month's change, by vendor, app and environment, and what drove it | Hands off to the jobs below | A budget per vendor and environment, lowered after verified savings | Usage × rate must reproduce the bill. A vendor that can't be read is BLOCKED, never $0. |
| Idle services | Idle for 7+ days in both host metrics and app logs | Sleep or stop it. Held: no new errors in callers; wake-up time acceptable. | A tripwire if a stopped service is redeployed | Idle in two sources |
| Oversized or misplaced resources | Real memory use vs allocation; a cache far from compute | Cap, change plan or move region on a canary. Held: slowest responses, errors and crashes over 48 hours. | A memory ceiling with automatic rollback | Target price checked the same day |
| Storage and backup bloat | Backups more than 3× the data; growth with no retention | A retention job. Held: a restore test passes and search stays up. | A growth tripwire | Delete counts dry-run on a copy first |
| Costly models | An expensive model on a high-volume feature | A per-feature model or effort flag. Held: no worse on replayed real requests, with speed held. | Allowed models per environment, a cost-per-task ceiling | Catalog price including cache and long-context surcharges |
| "Who spent this?" | Key spend minus what the apps recorded: the unattributed share | One capped key per app and environment. Held: no authentication errors. | A tripwire for a dev, test or CI process holding a prod key | Same time window in both sources |
| Dev, test and CI spend | Paid calls outside a budget; agent-created paid resources | Replay by default, cheap models in dev, an expiry date on everything agents create. Held: tests still catch a planted bug. | Caps per environment, a repeat-call breaker, an expiry reaper | Estimates cite the plan and the remaining allowance |
| Unused plans and credits | Paid plans with no use; under 14 days of credit left | Cancel or top up. Held: the feature still works. | A runway alert | Zero use in both vendor and app logs for 30 days |
| "What will it cost?" | An estimate from your plan, the price catalog and a small metered trial | Estimate tracked against actual | An unsourced estimate is BLOCKED | The price date is cited |
| Local disk and worktrees | Free space falling; merged or idle worktrees | Delete them. Held: no live session touched. | A free-space floor, worktree expiry | No session holds the worktree |

Every applied change keeps its undo in the receipt. Savings count only after the 7- and 28-day re-measure against a matched baseline, confirmed by the bill; until then they're INCONCLUSIVE. Deletes stay with a person.

**The infra map.** One picture of what runs, how it connects, how each part is configured, and what each part costs and does. It's built once, then rebuilt only when infra changes.

| Part | Where it comes from |
|---|---|
| What exists | What the repo declares: deploy configs such as Wrangler, Docker, Compose or Terraform files, CI workflows and database migrations. What is deployed, from each vendor's API: services, databases, buckets, queues, schedules, phone numbers and plans. The map shows where the two disagree, such as a test service that is still running. |
| How it connects | Bindings and calls from the configs, confirmed by the traces the app already sends. The traces also show which features use each part. |
| Config | Plan, region, limits, schedules, bindings, retention and the names of environment variables, never their values. Each change is kept as a diff, so a shift in cost or speed can be tied to the config change behind it. |
| Cost and use | Each part's daily cost from bills and usage APIs, its use (requests, compute time, storage, minutes) against its plan limits, and its share of the cost per successful task. |

The full discovery runs once. After that, three triggers re-map only the part that changed:

- A PR or merge that touches infra files. The PR also gets a cost and use diff before it merges.
- An agent running a deploy, migration or infra command, seen by the hook before and after each command.
- A daily check of each part's config fingerprint against the vendor's API, which catches changes made by hand in a console.

Cost and use are still read daily, because traffic moves them even when nothing is reconfigured. They're watched with control bands and no model. You hear about infra only in four cases: a new paid part appears, what's deployed differs from the repo in a way that costs money, a paid part has no owner or no traffic, or an infra change is over budget. Everything else sits in an Infra panel in the app. Where existing tools already read a vendor, the map uses them: Cartography and CloudQuery for cloud inventories, and Infracost for cost diffs on Terraform changes.

### The Money view

- **Home:** six numbers, each with a check badge (OK, STALE, DISAGREES, BLOCKED):
  - spend vs budget
  - verified savings
  - unattributed share
  - dev, test and CI share
  - credit runway
  - your hours on cost this week
- **Vendors:** bill, driver, source and last read per vendor, app and environment. A vendor that can't be read names the console step it needs.
- **Findings, changes and guards:** ranked by dollars per month × confidence, with approve, reject and roll back.
- **Dev and agents:** spend per worktree, test run, PR and workflow; resources agents created, and when they expire.

### No SDK for measurement

- **Bills and usage:** vendor APIs and exports.
- **Model calls:** gateway logs and the app's own ledger. Tokens come from OpenTelemetry, and prices from a catalog.
- **Agent spend and tool calls:** the agents' own transcripts, hooks and OpenTelemetry export. Claude Code's export already records each tool call with its duration and the time it sat waiting on you, and each model request with its tokens and cost.
- **Infra:** vendor usage APIs and host metrics, per service and environment per day. Per-request infra cost is allocated by traffic share only when a question needs it.
- **Your own app:** whatever it already records per call, in its own tables or OpenTelemetry spans. The pilot's booking app already wraps its agent's model stream to record each model and tool call's cost by task. It also logs every outside API call with its units and list price, and keeps a view of each task's outcome. So cost per confirmed booking needs no new code there. `dr` reads those tables with a read-only role and checks them against the vendors' own usage.
- **Attribution needs a convention, not an SDK:** one key per app and environment, three attributes on calls you already log (environment, feature, tenant), and one ledger row per paid call that isn't logged today. `dr` publishes this as a contract with conformance tests. An optional helper of about 50 lines covers apps that log nothing yet, with no proxy.

## The core everything is built from

The [core](#a-small-core-and-everything-else-as-an-extension) has six concepts and four invariants. The extensions describe their work with the fourteen terms and ten rules below, and each term maps to a core concept. Any tool or team can adopt them, because the formats are open. A **change** is measured with **readings** and checked against **specs** that someone owns. The tool runs the checks itself and records a **proof**, which ends in a **verdict** and a **receipt** tied to what ships. Every fixed failure leaves a **guard** for its whole class, and people get only the **decisions**.

## Fourteen terms the extensions use

### Measure

- **Reading** A measured number with a unit, a source, a freshness and a cost basis. There are four kinds: money (dollars, GPU-hours, quota, cost per successful task), speed (by stage, turns per task), time (rework, review wait, blocked time) and quality (evals, errors, retries). _Example: $0.019 per successful page task, from vendor usage, 2 hours old, at list price._

- **Ledger** The append-only record of every reading, proof, verdict, receipt, decision and open item. Everything else reads from it. Evidence (screenshots, videos, logs) is stored beside it on your machine, named by a hash of its contents so a receipt can't point at swapped files. It's kept for a set number of days, except evidence behind a decision, which is kept for good, and it's never committed to git. _Example: one local file per project, exportable. Screenshots go to your own bucket only if you set one._

### Define

- **Spec** What must hold. The tool drafts it from what the repo already runs, and its owner approves it once. It has a scope (the failure class it covers), an oracle (how truth is checked), an environment (what must be real), a budget (margin, noise floor, minimum runs), protection (who may change it) and a soak (how long to keep checking). _Example: the count on the page equals the count in the email and in history._

- **Check** An executable test of a spec, in the runner you already use. It must show that it can fail. A journey is a check written as a user task with an end state. _Example: a property test over random add, delete and retry sequences, or a checkout journey in a real browser._

- **Owner** The person who decides for an area, spec or guard, and approves changes to it. _Example: you for billing code, a teammate for their module._

### Prove

- **Change** Anything that can alter behavior or cost: a PR, a config or prompt edit, a model switch, a dependency or price change, including ones you didn't choose.

- **Trigger** What requires a proof: a change, an upstream change, a schedule, an incident or a broken budget.

- **Lane** Where a check runs, each with a price and a speed: local, CI, nightly, live or soak.

- **Proof** The record of running the checks for a change: before and after, the run count, the readings, and the environment's fingerprint.

- **Verdict** One of five outcomes: `PASS` `FAIL` `INCONCLUSIVE` `BLOCKED` `INVALID`

- **Receipt** A signed statement tying a proof and its verdict to the exact build that ships. Any CI or deploy step can verify it.

### Keep and decide

- **Guard** A spec that is enforced. It moves through observe, warn and block, and records how often it fires and how often it's wrong. Its owner promotes it, and it's retired only with a receipt.

- **Decision** A call only a person should make: taste, a trade-off, an approval or an accepted risk. It goes to the owner and is saved as a baseline or a rule.

- **Open item** Something unresolved: it needs you, it needs a run, or it's a gap. It blocks "done" until it's resolved or accepted.

## Ten rules that keep the evidence honest

1. **The tool owns the proof.** A check must fail before the fix and pass after. What the agent says about its own work is never evidence.
2. **Guard the class, not the instance.** A guard for the last bug doesn't catch the next one. Group repeated failures and write one spec for the class. When two fixes that share an assumption fail the same check, the next step questions the assumption instead of adding a third patch.
3. **Missing is never passing.** A missing environment, access grant or approval gives BLOCKED, with a cause, an owner and an age. It is never PASS and never a silent skip.
4. **Nothing passes silently.** Every check reports what it actually ran and when it last really passed. A green check that ran nothing is INVALID.
5. **Re-prove on every trigger.** That includes changes you didn't choose: a model retired, a CLI updated, a price changed. When a preview of the new version exists, run a canary against it before the switch date.
6. **Checks are protected, and they must bite.** Agents can't edit checks, expected outputs or baselines. Each check is shown to catch a planted copy of the bug, and a real past mistake when the ledger has one.
7. **A fix reaches every copy.** It isn't done until every branch, release and install that needs it carries it. Backport PRs carry the spec, and a version check at session start updates agent tooling.
8. **Cheapest lane, enough runs.** Run each check where it can still show the failure, as many times as the accepted failure rate requires. Five clean runs can't rule out a 45% failure rate; thirty bring it near 10%. Too few runs give INCONCLUSIVE. If the same build, spec and environment already have a receipt, reuse it instead of rerunning.
9. **Check the reading before acting on it.** No cap, rollback or optimizer fires on a number that is stale, on the wrong cost basis, or contradicted by a second source. The check is a tool any automation can call. Destructive actions also get a preview of who's affected, a dry run, and a rollback that was rehearsed recently.
10. **One reading improves only if the others hold, and people own the trade-offs.** Budgets only tighten unless their owner approves. Taste calls go to people, and every decision is saved so it's never asked twice.

The core enforces rules 1, 3 and 4, the protection half of rule 6, and the "never asked twice" half of rule 10. The rest are defaults of the extensions that need them.

## Blocked time gets measured, not shrugged off

Waiting on access is often the largest friction in a big organization, and nothing counts it. A tool can't grant access, but it can record every blocked event with its error signature, cause, owner and hours lost, fix the part you can fix yourself, and hand the rest to the owning team with evidence.

| Cause | Signals | Who can unblock | What the tool does |
|---|---|---|---|
| Sign-in and expired tokens | 401s, sign-in prompts, MFA loops, token lifetimes | You, or the identity team | Checks token lifetime against the planned run before it starts; registered test identities refresh themselves |
| Permissions and approvals | 403s, pending requests, time from filing to grant | The resource owner | Drafts the request early and complete; reports wait time by owner |
| Quota and rate limits | 429s and their headers | You, or the platform team | Tells a short cooldown from spent quota and from provider capacity; paces runs and resumes after the limit window |
| Network paths | Refused or unreachable destinations | The network team | Derives required destinations from code, config and traces, and drafts the request before merge |
| Environments | Provisioning failures, queue waits, version mismatches | The platform team | Declares what each run needs and gives BLOCKED with the exact gap |
| Local setup and broken tools | Full disks, busy ports, missing credentials, tool crashes | You | Doctor preflight, plus scripted fixes for known errors |
| Policy rules | Denied tool calls | The policy owner | Replays recorded calls against a proposed rule to show which workflows it would break |

Each blocked event is a BLOCKED verdict with a cause, an owner and an age. The weekly total is a reading under "your time".

## How things move

- **A change:** proposed → checked → verdict → receipt → merged → released to each channel → checked live → soak → re-proved on each trigger

- **A guard:** failure class found → proven to bite → observe → warn → block → retired with a receipt

- **An open item:** raised by an agent or check → needs you · needs a run · gap → resolved, accepted by owner, or filed

- **A blocked check:** BLOCKED, with cause and owner → aged and counted as blocked time → unblocked, then re-run

Every promotion of a guard and every acceptance of an open item is a decision, made by the owner and saved in the ledger.

## Three presets

A preset is a list of extensions plus settings, not a separate product.

| Setting | Personal | Team | Enterprise |
|---|---|---|---|
| Who approves rules and budgets | You | The owner of each area | The owner, through a proposal; hooks arrive through managed settings |
| What a receipt can do | Gate merges | Gate merges in owned areas | Shorten human review; never unlock an automatic merge |
| Capture | Local, redacted | Local, shared ledger opt-in | Opt-in, content-free, pseudonymous; each person's numbers are shown only to that person |
| Incidents | Automatic rollback for quality | Same, owner notified | Rollback for quality; freeze and preserve for suspected security events |
| Money readings | Dollars from bills | Dollars per team and environment | Quota, capacity and dollars, on contract rates |

## What other tools can produce and check

Teams adopt formats faster than services, and formats outlive their maintainers. Four formats are published: the spec file, the verdict names, the receipt, and the guard capability list. Any test runner, CI system or hook runtime can write or check them without running DoneRight.

### Spec

```
spec: count-agreement@3
claim: For any user and period older than 10 min,
       page count = email count = history count = event log count
scope: class "counts disagree across surfaces"
oracle: the event log, not a shared counter
environment: { db: masked-snapshot, email: recorded, clock: pinned }
budget: { mismatches_per_day: 0 }
checks:
  - property: random add/delete/retry/duplicate sequences, 500 cases
  - monitor: hourly reconciliation on live data
protection: { agents: read-only, owner: you }
soak: 28d
```

### Receipt

```
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [{ "name": "web-app", "digest": { "sha256": "9f2c…" } }],
  "predicateType": "urn:doneright:proof:v0.1",
  "predicate": {
    "change": "PR 482",
    "verdict": "PASS",
    "specs": ["count-agreement@3", "page-task-speed@2"],
    "proof": { "fails_before": "a1b2c3d", "passes_after": "4e1c9ab",
               "runs": 30, "failure_rate_bound_95": 0.10 },
    "checks": [{ "name": "journey:read-page@2", "runs": 30, "clean": 30,
                 "known_failures": [],
                 "end_state": [{ "assert": "summary matches the page",
                                 "selector": "#summary", "box": [24, 310, 612, 96],
                                 "pass": true }] }],
    "evidence": [
      { "sha256": "91d2…", "type": "screenshot", "check": "journey:read-page@2",
        "step": "end state", "run": 1 },
      { "sha256": "c4e8…", "type": "screenshot", "derived_from": "91d2…",
        "made": "highlight, on request" },
      { "sha256": "77ab…", "type": "video", "check": "journey:read-page@2", "run": 1 } ],
    "readings": { "p95_s": [22.0, 8.1], "success": [0.94, 0.95],
                  "usd_per_task": [0.031, 0.019] },
    "environment": { "parity": "match", "fingerprint": "model+sdk+lockfile+agent-config" },
    "open_items": []
  }
}
```

Every screenshot, video and trace is listed by hash, with the check, step and run it came from, so changing any of them breaks verification. A highlighted copy, made on request, points back to its original.

### Verdict names

- `PASS` Proven, and every reading held over enough runs.
- `FAIL` Something got worse; the smallest failing case is attached.
- `INCONCLUSIVE` Too few runs to tell; the number needed is attached.
- `BLOCKED` Something outside the code is missing; the cause, owner and age are attached.
- `INVALID` The check itself is broken, can't fail, or ran nothing.

### Guard capability list

What any hook runtime needs in order to host a guard:

- Observe, warn and block modes, switched by the owner.
- Counts of fires, breaks and reported false positives.
- A known-good and a known-bad example that are tested on install.
- Install and uninstall at user level only, never written into a repo's settings.
- A clear message telling the agent what to do instead.

## Each problem, what the product does, and when

| Problem | What the product does | You know it works when | Release |
|---|---|---|---|
| "Is it really done?" | A done gate runs your definition of done whenever an agent says done or opens a PR. The tool owns the proof and the verdict. Open items are tracked until each is resolved or accepted. | "Is it done?" follow-ups fall by half | R1 |
| The issue already said what done means | The done gate reads the issue's acceptance criteria and matches each change to a sentence in it. The verdict decides whether the PR says "Closes" or "Part of". In the pilot's last two weeks, 134 of 243 sessions started from an issue link alone, and 101 of them later needed a "done and validated?" check. | Sessions that start from an issue need no follow-up about scope or closing | R1 |
| "It passed on fakes" | Journeys run on a real stack and check the end state. Stand-ins are detected, and each claim needs a minimum lane. See [E2E proof](#prove-it-the-way-a-user-would-use-it). | No claim a user would notice is proven only on stand-ins | R1, R3 |
| "What is each agent doing, and what does this mean?" | `dr status` shows each running agent as working, waiting on a lease, waiting on you, or stuck. Every change, verdict and open item carries a three-line plain summary. | "Still going?" and "explain this" asks fall by half | R1 |
| "Start the local stack" every time | `dr env up` brings up the stack the proof needs, one per worktree with its own ports, after a doctor preflight. It stops only the processes it started. | No manual stack starts; no colliding stacks | R1 |
| Checks that pass without running | Gate health: each check reports how many tests actually ran and its last real success. Stand-in environments get BLOCKED. | Zero silent gates | R1 |
| History disappears | Transcripts and CI results are archived before their retention runs out, with an alert if capture goes quiet. | Nothing lost to retention | R1 |
| Waiting on access | Every blocked event is recorded with its signature, cause, owner and hours lost. The tool fixes the self-serviceable part, such as expiring tokens and known setup errors, and drafts complete requests early for the rest. | Blocked hours visible weekly by cause and owner, with the top cause shrinking | R1 |
| Spend is invisible, and cost fixes come undone | Bills and usage for the vendors you use, checked readings, ranked findings, changes with proof and guards. See [Money and infra](#finops-built-on-the-same-loop-run-by-agents). Billing emails and every other vendor come later. | Verified savings that stay; your hours on cost going down | R2, R3 |
| Automation trusts a wrong number | Checks each reading before any cap, rollback or optimizer acts. | Zero automated actions on an unchecked reading | R2 |
| Models and tools change under you | Watches models, CLIs, SDKs, gateways and prices, and re-proves the affected changes. | Every upstream change is followed by a fresh verdict | R2 |
| Things got slower | Speed by stage, compared with the version before the change. | Slowdowns are caught before users report them | R2 |
| Alerts are noise | Control bands on a rolling baseline replace fixed thresholds. By size, each breach is logged, diagnosed, or answered with a PR, and it arrives with its evidence. | Every alert you see comes with a diagnosis, and dismissed ones stop coming back | R2 |
| Cost cuts are risky | Model, routing, caching and plan changes ship only with a PASS on quality and speed, then roll out gradually with rollback. | Dollars saved with quality held | R3 |
| Fixed bugs come back | A guard for each failure class, proven to fail on the old code, protected from agents, and measured once it's live. | Guarded classes stay away for 30 days | R3, R4 |
| Merged isn't shipped | Each change moves from merged to released in every channel to checked live. Checks deferred until "after deploy" reopen when the release lands, and a release that skips a channel leaves the fix BLOCKED there. | No fix is called done before it reaches the users who reported it | R3 |
| Fixes don't reach every copy | Tracks which branches, releases and installs carry each fix, opens backport PRs with the spec attached, and checks agent tooling versions at session start. | Every fix reaches its copies | R4 |
| Your own agents step on each other | Every session knows who else is in the repo, what it changed and what it holds. An agent hears before it edits a file another session changed, or when a dry merge shows a conflict, and it's told when a sibling's merge touches its files. Migration numbers, ports and test accounts can't be taken twice, and two sessions on one issue become one question to you. Killing by name, a shared stash and git aimed at another worktree are blocked from R1. | No killed stacks, overwritten setups or duplicate fixes between your own sessions | R1, R2 |
| A plan limit runs out mid-task | Each account's 5-hour and weekly windows and each task's tokens, from the agents' own data. Then tasks are scheduled into the windows, with a share kept for you. | No task stalls on a limit you didn't see coming | R2, R6 |
| The same workflow, driven by hand every time | Your usual steps run as a playbook drafted from your history. Each step is gated, each task goes to the agent and account that fits, and every change to a flow is replayed before it applies. | A one-line request becomes a proven first release, and you're asked only taste and approval calls | R6 |
| Teammates overwrite each other | An ownership check before an agent edits a teammate's recent lines, a claim on each issue before an agent starts, early warning on overlapping work, and a check of the merged result that re-runs recent fixes' tests. | Overwrites and duplicate fixes near zero | R4 |
| Main moved again | When main moves, the agent rebases its branch, re-runs the affected checks and refreshes the verdict, so you never paste a conflict banner back to it. | No PR waits on you for a rebase | R4 |
| Agents stop to ask what you already answered | A policy check runs whenever an agent stops with a question, asks one, or hits a permission prompt. Covered questions are answered and the agent goes on. Answers you keep repeating become proposed policy. | Questions with a standing answer never reach you | R1, R4 |
| The same error keeps costing time | Every failed call is recorded by signature. A recurring one gets a known fix applied before it fails again, a root-cause change with proof, or a BLOCKED item for its owner. | No error signature keeps coming back | R1, R4 |
| Agents overbuild | A size check on every claim, against the plan and your past changes; plans sized before code; anything new the plan doesn't name is a decision | A fix stays the size of the problem | R1 |
| Agents forget or ignore what you told them | Reads every agent's memory and transcripts. Checkable lessons become guards, the rest become one rule set written into each agent's instruction file, and you answer keep or remove from a ranked list. | No correction has to be given twice, to any agent | R4 |
| A rule change made agents worse | Every change to instruction files, skills, hooks, guards or policy is replayed against real past tasks from your transcripts before it lands. | No configuration change lowers the pass rate unnoticed | R4 |
| CI is flooded and red | When CI runs, caps per agent, flaky/stale/real labels, all tested on a replay of archived CI history. On a main that's already red, each failure is marked as new or inherited, and each job's flake rate is a reading. | Every red run is labeled real, flaky or inherited within an hour | R5 |

## What breaks in two of the busiest agent-built projects

OpenClaw and hermes-agent are two of the most-starred projects on GitHub, and agents write much of their code. We matched every commit to main from September 14 to early October 2026 against their issues and pull requests: 11,618 commits in OpenClaw and 13,569 in hermes-agent. Both ship 500 to 650 commits a day and merge almost everything without a human review, so their users are the last test. What breaks there at scale is what this product has to handle, and some of it changes the design.

| Pattern | What we found | What DoneRight does |
|---|---|---|
| Main is the test bed | In hermes-agent, 82% of merged PRs are merged by their own author and 85% get no review. Main is red about a quarter of the time, and users report regressions within hours. In OpenClaw, the hourly full CI run on main failed 186 of 222 times, and fixes for a red main go straight to main ([fd5331f](https://github.com/openclaw/openclaw/commit/fd5331f17e0854d47d0bfe158359ac8613384249)). | An after-merge mode for teams like these. Each red build or user report is tied to the push behind it, a revert is proposed with a receipt, and "done" is judged at release. On a main that's already red, each failure is marked as new or inherited. |
| Checks that ran nothing | hermes-agent's live provider canary reported success with every real step skipped because no key was set, including on a release gate ([run](https://github.com/NousResearch/hermes-agent/actions/runs/35985781919)). 12.7% of pushes to main got a CI run with zero jobs. | A skipped check is BLOCKED, never PASS (rules 3 and 4). |
| Tests written in bulk, deleted in bulk | hermes-agent deleted about 8,200 tests in one PR ([#120071](https://github.com/NousResearch/hermes-agent/pull/120071)), then restored the 175 that fail when their fix is reverted ([#120220](https://github.com/NousResearch/hermes-agent/pull/120220)). OpenClaw ran about 180 batches removing low-value tests. | Tests must bite before they land, so they don't need bulk deletion later. A deletion needs a receipt. |
| Taste calls landed again | One hermes-agent design was merged and reverted three times, starting with [#112669](https://github.com/NousResearch/hermes-agent/pull/112669) and [#113037](https://github.com/NousResearch/hermes-agent/pull/113037). About 3,000 open items there are labeled `needs-decision`. OpenClaw's stale bot closed 242 items that were waiting on a product decision. | Decisions are saved and enforced, so an agent reviving a rejected direction is stopped. A decision never expires silently; it ages and escalates. |
| Duplicate fixes | 31% of hermes-agent PRs that link an issue target one that already has a PR. One bug drew six outside PRs ([#128974](https://github.com/NousResearch/hermes-agent/issues/128974)). In OpenClaw, 179 issues drew two or more fix PRs. | A claim on each issue or failure class before an agent starts, and a warning when work overlaps. |
| Stale merges undo fixes | A 234-file hermes-agent PR built on an old copy of main deleted a fix and its tests, and the fix missed two releases ([#126492](https://github.com/NousResearch/hermes-agent/issues/126492)). | The merge check re-runs recent fixes' tests on the merged result. |
| Fixed on main, not for users | OpenClaw 2026.9.8 shipped without a Windows fix merged two days earlier ([#162332](https://github.com/openclaw/openclaw/pull/162332)), and a later report was closed as fixed on main ([#164945](https://github.com/openclaw/openclaw/issues/164945)). | A change moves from merged to released in each channel to checked live. A skipped channel is BLOCKED. |
| Instructions act as configuration | After OpenClaw added an AGENTS.md rule on test cost, the share of PRs that state their test cost went from 8% to 56% within a week. The file is trimmed to stay under a 20K bootstrap cap ([#163325](https://github.com/openclaw/openclaw/pull/163325)). | An edit to an instruction file is a change, with readings before and after. |
| CI cost reshaped CI | In OpenClaw, about 75% of full CI runs on main were cancelled by newer pushes, so full CI moved to hourly. In hermes-agent, 57% of runs on main are cancelled. | Cancelled-run waste is a reading, and a change to CI policy is replayed on archived history first. |

The counts come from public GitHub data, through clones without file contents and the GitHub API. Neither project's agent share can be measured directly, and links from a fix to the change that caused it are estimates. The linked items were checked by hand on October 5. Practices worth copying are in [Borrowed practice](#what-we-copy-from-teams-already-doing-this).

## Proof of "done" first, then everything else builds on it

Each column is one week, starting Monday, October 5. Decision 1, in mid-November: does the done gate cut your "is it done?" checks, and do you answer taste calls from the inbox instead of chat? The open-source launch follows R2. Decisions 2 and 3, in early March: keep going, narrow or stop the open-source tool, and whether the enterprise pilot earns a paid enterprise version.

### R1 · Oct 5 – Nov 8

**Prove "done", and see it.** This is where your time comes back first. The core ships here with its first extensions: the Claude Code and Codex adapters, your wiki, the done gate with specs drafted for you, the app and browser journeys. The pi adapter and a clean-machine install follow in R2.

**What you see**

- A terminal view for numbers and follow-ups: `dr status`, `dr inbox`, `dr show`.
- The app, one native desktop app, for screenshots side by side, one-click answers to taste calls, open items and charts, in three views: Needs you, Work and Numbers, plus a box where you say in words what to change. It sets itself up on first launch, with no setup screen. The menu bar shows one count, notifications carry the answers, and the hub starts at login. `dr open` opens the app at what needs you, or at a session, change or report. No hosting. Each taste call shows your past calls on the same screen, so you judge with your own history in view.
- Each verdict is a GitHub commit status that branch protection can require. A PR comment with screenshots and `dr report`, one HTML file per change, follow in R2.
- `dr status` and the Work view list every session from every agent and repo, built for dozens at once: grouped by repo and issue, with what needs you and what's stuck first, and quiet sessions folded into one line. They flag two sessions on one issue, a session an app quit or a usage limit interrupted, and an idle session with work that isn't pushed. From any session's row you can open it in its app, pause it, or send it a note: a note reaches a Claude app session right away and a Codex app session when it next stops. Starting, steering and stopping sessions come with the ones DoneRight starts itself. A Map switch shows the same work per project: each part, from the repo's own structure, with the sessions on it now, its open issues and PRs, and its evidence, the tests and journeys with their last result, plus the parts nothing proves yet. Every change, verdict and open item has a three-line plain summary.
- Later, an optional extension can read an [Understand-Anything](https://github.com/Egonex-AI/Understand-Anything) graph (`.ua/knowledge-graph.json`) when a repo has one: its parts, layers and dependencies fill in the Map; what it says a change affects helps pick the checks and journeys to run, and flags a change that reaches parts the plan doesn't name; its domain flows can seed the Flow view; and it can draw your wiki as a graph. Its first run uses many tokens, so DoneRight never starts it on its own; it runs only when you ask, on your plan. Our own maps take three lessons from it: graphs that teach rather than impress, tours ordered by dependency, and detail that adapts to who is looking.

**Your time**

- Rework, review wait, "is it done?" asks, agent spend and blocked time, from agent sessions, git and CI. GitHub Actions and GitLab CI are both supported.
- Lifecycle readings come built in: time from issue to merge, first-pass merges, rework cycles, time to first review, time waiting at each approval, and the DORA measures.
- Every blocked event is recorded with its error signature, cause, owner and hours lost. Causes include sign-in, expired token, approval, quota, network path, environment and local setup.
- Failed tool calls are recorded the same way, and known fixes are applied before the same error repeats.
- Capture can be content-free: counts, signatures and durations, never text.

**Done gate**

- Your definition of done is drafted from what the repo already runs and what you've asked agents before, and you approve it once. Not happy with a line? Say what to change in your own words, in the app or to your agent; it comes back as a diff, and nothing is written until you apply it. It runs whenever an agent says done or opens a PR. It reads the issue's acceptance criteria, and the verdict decides whether the PR says "Closes" or "Part of". When the agent wrote a plan, the gate runs the plan's proof steps and compares the diff with it.
- The tool owns the proof and gives a verdict. Open items are tracked, and only taste calls come to you. A claim that stalls, with the same open items twice and no new evidence, stops going back to the agent: it comes to you once, as a decision to split the issue, fund the live runs it needs, or accept what's proven. Every claim also gets a size check, with no model: lines, files, new dependencies, folders or services, files outside the plan, code no test reaches and duplicated blocks, against the plan and what you've accepted for the same kind of change in that repo. Over that, the agent hears it first and trims or says why; still over, it's one taste call for you. Size alone never fails a change.
- A policy check answers the questions your instruction files and saved decisions already cover, whether the agent stops with a question, asks one or hits a permission prompt.

**Environment up**

- `dr env up` starts what the proof needs from your repo's own scripts or compose file, with one stack per worktree on leased ports.
- It tracks every process it starts and stops only those, by PID. It checks health before the proof runs and tears everything down after.
- The environment manifest leases test accounts, phone numbers, simulators and screens to one session at a time. Each real resource has a cap, a cooldown and a cost estimate before every run. You approve a spending budget once per issue, with runs, a dollar cap and an end date, and every run inside it goes ahead without asking.
- The setup is learned once into your wiki, from your scripts and from how past sessions started the stack, so you never tell an agent how again. Credentials are named by variable and source, such as a dotfile or a CLI login, and loaded only into the process that needs them. The agent never sees a value or a path.

**Journeys**

- Browser and API journeys first, checking the end state on a real stack, with video and screenshots in the ledger.
- The agent's test plan is checked by the tool before tests are written: real services on the claimed path, a test that can fail before the fix, and a cost within budget. On a bigger issue the plan also names the files and parts it will change, and a plan past the repo's usual size comes to you once, before any code: approve, shrink or split. A dependency, service or top-level folder the plan doesn't name is a decision, answered by your standing rule or asked.
- The final check runs in a context that didn't write the code, and covers the change plus its nearest neighboring flows.

**Trust the record**

- Gate health: did each check really run?
- An archive that keeps transcripts and CI results before they're deleted.
- A "doctor" that checks sign-ins, token lifetimes, disk and ports before a run, and applies scripted fixes for known errors.
- A key pasted into chat opens a "rotate this key" item.

**Done when** "is it done?" follow-ups fall by half, and you answer taste calls and open items from the inbox instead of in chat.

### R2 · Nov 2 – Dec 6

**Find, and trust the numbers.** Before anything acts on its own, the readings and verdicts have to be trustworthy.

**Verdicts**

- Defines what "held" means for each reading: a margin, a noise floor and a minimum number of runs, with versioned metrics.
- Run counts come from the failure rate you'll accept: 5 clean runs can't rule out a 45% failure rate, 30 runs bring it near 10%.
- On a main that's already red, each failure is marked as new or inherited, and each job's flake rate is a reading.

**Checked readings**

- A reading is checked for freshness, cost basis and a second source before any cap, rollback or optimizer acts on it.
- A meter auditor cross-checks every meter you rely on. Any automation can call the same check and get OK, STALE, DISAGREES or WRONG BASIS back.
- Destructive actions get a preview of who's affected, from two sources, and a dry run first.

**More agents and reach**

- The pi adapter, then Cursor.
- A clean install, two ways: the app download, which carries the hub, the hook client and the `dr` command, so people who start from the app never need npm; or npm, with prebuilt binaries and a Claude Code plugin, for terminal-first installs and CI.
- The line under the menu bar, in the app: what needs you hangs on a line under the menu bar, shown when the pointer rests at the top edge or with a shortcut, and you answer it in place.
- Add anything: type what you want in the app's box, the same one where you change a rule or remove DoneRight from an agent, or run `dr extend "…"`. Your own agent, Claude Code or Codex, builds it headless on your own sign-in and plan. DoneRight has no model or API key of its own. Each build shows the plan usage it took, and builds wait when the plan window is down to your reserve. The agent writes an extension package: panels and cards for the app, watchers and schedules, a manifest of what it may touch (folders, the clipboard, the screen only while a shortcut is held, a small window by the cursor, notifications, hosts, credentials by name, schedules, the agent), tests and a README. Before you see it, it passes the conformance suite, its own tests, a headless render of each panel, a dry run on your recent data, and an access check that it touched only what it declared. A failure goes back to the agent like any other claim. One approval shows what it can and can't do. It installs pinned, `dr off` covers it, and removing it leaves no trace. A generated extension can't pass a check, write a decision or answer an ask. Three examples: "Hang every screenshot I take on a line under the menu bar", "When I hold ⌥Space, look at my screen and point at what I ask about", and "Watch the receipts inbox and tell me when a receipt bounces". An extension can have your agent look at the screen and point, or act on it with the agent's own computer-use tools. Anything outward-facing, such as send, buy, post or delete, is an ask, and every action is recorded with screenshots.
- What you add improves with use. DoneRight records, on your Mac only, how each extension is used: opens, dismissals, undos, errors and gestures. When a pattern shows up, it proposes a change in plain words, your agent builds it, and the change is replayed against your recorded uses. A change that alters a past result is held for you. A change that stays inside the access you already granted can apply on its own once you've approved three alike, with one-click undo. New access always asks.
- Try any extension side by side before you switch: say "try Hindsight as my memory", or run `dr try <extension>`. A copy runs next to the one in use, gets the same events and keeps its own store, but only the active one reaches agents. After a while, a week by default, DoneRight replays your past sessions against both and compares them. For memory, that's the facts you had to repeat to agents that each would have supplied at session start, wrong or stale facts, the tokens each adds to a session start, latency, and any cost, since Hindsight makes its own model calls. You get one decision: switch, keep both, or remove the copy, and switching back is instant because each keeps its own store. It works for any extension point (memory and indexes, check runners, schedulers), and it's how you pick a memory module: the default wiki index, Hindsight or a graph memory.
- A PR comment with the verdict and screenshots, and `dr report`, one HTML file per change.

**Infra cost**

- Bills and usage for the vendors you actually use, through their APIs, each with a check badge.
- Findings ranked by dollars per month: idle services, oversized resources, backup bloat, unattributed spend, dev and test spend, unused plans, credit runway.
- The infra map: one full discovery, then re-mapped only when infra changes. It's drawn as a layer on the project map, so each part shows the services and vendors it uses.

**Parallel sessions**

- Every session knows who else is working in the repo: which agent, which issue, which files it changed, and which ports and test accounts it holds.
- An agent hears about a collision before it happens: when it's about to edit a file another session changed, or when a dry merge shows two branches will conflict. When a sibling's change merges and touches its files, it's told to rebase before it says done.
- What can only be used once can't be taken twice: a migration number another branch already took, a port or a test account. Two sessions on the same issue become one question to you.
- Each notice is built from facts such as paths, issue numbers and line ranges, never from another agent's words, so one agent can't steer another.

**Plan limits**

- Each account's 5-hour and weekly windows and their reset times, read from Claude Code's and Codex's own data.
- Tokens and window share counted for every task, with what the same work would cost on an API key.
- DoneRight never signs in, switches accounts or touches credentials. Sign in to another account and the work carries on with that account's windows.

**Find**

- Speed by stage. Each spike is tied to the deploy, PR and session behind it, and alerts arrive with evidence and a proposed fix.
- A watcher re-proves affected changes when a model, CLI, SDK or price changes. When a preview of the new version exists, an upstream canary runs your specs against it before the switch date and attaches the evidence to any overlap or rollback request.
- An after-merge mode for teams that push to main all day: each red build or user report is tied to the push behind it, and a revert is proposed with a receipt.
- Alerts use control bands on a rolling baseline instead of fixed thresholds. Detection uses no model. By size, a breach is logged, diagnosed read-only, or answered with a PR or a pre-approved runbook, and dismissals with a reason tune the bands.

**Done when** no automated action uses an unchecked reading, and too-small samples come back INCONCLUSIVE instead of PASS.

### R3 · Nov 30 – Jan 17

**Change with proof.** Agents cut cost and fix slow steps, and every change carries a receipt.

**Changes**

- Cheaper default models and routing first, since they save the most. Then caching, batching and fixes for slow steps.
- Infra changes: sleep idle services, rightsize, add retention, split keys per app and environment. Each comes with a dry run, an undo in the receipt, and savings re-measured at 7 and 28 days.
- Rollback is rehearsed on a schedule. An automated action that relies on a rollback nobody has rehearsed recently is BLOCKED.
- A PR that changes infra gets a cost and use diff before it merges.

**Valid environments**

- Each test declares which dependencies are real, recorded or stubbed, and the tool compares that with what ships. Each run gets throwaway test data, and test runs can't write to production.
- Stand-in detection: a test that still passes with its real dependency switched off can't prove a claim users would notice.
- Desktop, phone, text-message and chat-app journeys on leased real accounts, in a live lane that runs only when asked or before a release.

**Tests that bite**

- A new test must fail on the old code, catch a fault planted in the changed lines, and name the journey or failure class it protects. Otherwise it's INVALID.
- A test is deleted only with a receipt showing every journey is still covered.

**Receipts**

- Tied to the exact build in a standard attestation format, with a small verifier any CI or deploy step can run. Receipts can be written into an evidence ledger you already run. Each budget and test ships with the change it protects.
- A proof cache: if the same build, spec and environment already have a receipt, it's reused instead of rerun. Run planning computes the fewest runs that can show the margin you need, starting in the cheapest lane.
- Each change moves from merged to released in every channel to checked live. Checks deferred until "after deploy" reopen when the release lands, and a waived release step is recorded as a decision.
- Receipts record the instruction files, skills, hooks and guards in force, so a regression can be traced to a configuration change.

**Done when** most changes ship with a receipt, and the monthly brief shows dollars and seconds saved with quality held.

### R4 · Jan 4 – Feb 21

**Keep.** Guards stop whole classes of failure and earn their place with numbers.

**Guards for classes**

- Repeated failures are grouped into classes. Each guard is proven to fail on a deliberately broken example first, and agents can't edit it.

**Measured rollout**

- Each guard tracks how often it fires, breaks or is wrong, and moves from watching to warning to blocking with its owner's sign-off. It's compiled into each agent's own hooks.
- A guard starts blocking only after a replay of past changes shows it's precise enough. Its switch is read from main, so a change can't relax its own guard.

**Teams and live runs**

- Team guards: R2's parallel-session notices extended across teammates' machines, a claim on each issue before an agent starts, and a merge check that re-runs recent fixes' tests on the merged result.
- When main moves, the agent rebases, re-runs the affected checks and refreshes the verdict.
- Review findings become open items ranked by severity. The agent that wrote a change can't approve it, and each reviewer's precision is measured.
- Registered test identities that refresh themselves.

**Agents use the app**

- On a schedule, an agent uses the app on the real stack and files what it finds, with video. Each confirmed bug becomes a journey.
- Journeys are also drawn from real usage, such as session replays.

**Fix reach**

- Shows which branches, releases and installs carry each fix, and opens backport PRs with the spec attached.
- A version check at session start updates agent tooling.

**Cost guards**

- Tripwires for a stopped service being redeployed, a prod key in dev, test or CI, agent-created paid resources without an expiry, low credit runway, and a disk below its floor.

**Memory, policy and fixes**

- Reads the memory and instruction files of Claude Code, Codex, pi and Cursor, plus their transcripts.
- Turns each lesson into a guard, a shared rule or a removal, ranked for you to keep or remove.
- Your “too complicated” remarks and reverts, and the change sizes you accept, become each repo's size budget.
- Answers you keep repeating become proposed policy. After the same kind of taste call gets the same answer three times, a rule is offered, bounded by what the recorded screens can check, such as the total and the pay button staying in view; with your yes, the next ones are decided for you, kept with their screenshots and an overrule. Prices, totals and anything outward-facing still always come to you. Recurring errors get fixes, measured and promoted like guards.
- Saved decisions are enforced: an agent reviving a rejected direction is stopped, and an unanswered decision ages and escalates instead of expiring.
- Every change to instruction files, skills, hooks, guards or policy is replayed against 20 to 50 real past tasks taken from your transcripts before it lands. Each incident adds a task.

**Done when** guarded classes stay away for 30 days, each guard's false-positive rate is known, overwrites and duplicate fixes are near zero, and no correction has to be given twice.

### R5 · Feb 1 – Apr 4

**CI.** CI costs money and time, so it gets the same treatment, using the history archived since R1.

**Policies**

- Full CI only when a change is ready, caps per agent, and no duplicate runs across worktrees.

**Triage**

- Every failure labeled flaky, out of date or real, with evidence. A test is deleted only with a receipt showing the behavior is still covered.
- Cancelled-run waste and each job's flake rate are readings.

**Proof first**

- A replay of archived CI history shows what a policy saves, and whether it would miss a real failure, before you turn it on.

**Done when** CI minutes per merged change fall, every red run is labeled real, flaky or inherited within an hour, and the replay misses no real failures.

### R6 · Feb 22 – Apr 4

**Run your workflow.** The steps you drive by hand on every project, from research through issues, builds, proof and the PR to the release, run as a playbook. You're still asked whatever needs a person.

**Playbooks**

- `dr create` turns a one-line request into your usual steps, each an agent session with a gate: research, issues with dependencies, builds in parallel worktrees, proof, the PR and the release.
- A gate is a check that runs plus something a named person accepts, such as a spec, a plan or a PR. A model reading the transcript is never a gate.
- The playbook is drafted from your own history, the steps you actually repeat, and you approve it once, like the spec.
- You start it. Taste calls, approvals and anything only you can unblock still come to you.
- The app's Flow view draws each playbook as one map, with every task moving along it. A playbook can be any work your agents do, not only code, such as a weekly report or booking travel.

**Within your plans**

- Tasks are scheduled into each account's windows, with a share kept for your own work. Big jobs start after a reset, and a task that won't fit waits instead of stalling halfway.
- Each task goes to the agent, account and model that fits: allowed for this repo first, then past results on this kind of task, then room left, then the cheaper one on a tie. The reason is shown.
- Sessions start through the unmodified Claude Code and Codex on your own sign-in. Nothing routes requests through a plan, and each scheduled automation is capped or moved to an API key.

**Learn from results**

- Every flow, skill, agent and model is scored by outcomes and your feedback: merges without rework, rounds to a pass, review rounds, your overrules, and passes you later reopened as broken. How often you accept an agent's work counts as sentiment, not quality.
- Only runs you accepted are learned from. A change to a flow, prompt or skill is proposed as a diff, replayed on held-out past tasks for each model, and applied only with your OK. Nothing rewrites itself.

**Done when** a one-line request becomes a proven first release with you asked only taste and approval calls, and no task stalls on a limit you didn't see coming.

## Same core, different profile

In a large engineering organization the same problems show up in different places. Writing code takes minutes and review takes hours. Access, not code, is the biggest source of waiting. Model and tool changes arrive from a central team. So the enterprise track runs the same product with a stricter profile and starts with what large organizations can't see today.

### What's different at scale

- **Waiting on access is the biggest friction:** sign-ins, environments, approvals and quotas. Blocked time by cause and owner becomes a headline reading.
- **Money arrives as quota and capacity,** not invoices, so the meter counts GPU-hours, runner minutes and reserved quota.
- **Model and tool changes are imposed,** so every one triggers a re-proof, and the evidence goes with any rollback request.
- **Platforms for sandboxes, policy, evals and telemetry already exist,** so we connect to them with adapters and don't replace them. Receipts go into the evidence ledger the organization already runs.
- **Access can't be granted by a tool, but it can be measured.** Blocked hours by cause and owner come with the evidence the owning team needs. Requests are drafted early and complete, and a policy simulator shows which workflows a proposed rule would break.
- **Teams adopt formats faster than services.** The verdict names, the receipt format and the guard list are published so any team can produce and check them.

### The pilot measures what's missing

- Blocked time by cause and owner, keyed on error signatures.
- Gate health: zero checks that pass without running.
- Fix reach: which branches, releases and installs carry each fix.
- Cost and latency per successful task, with readings checked before anything acts.
- Measured guards in the hook system the organization already runs.

It runs on repos that already have commit and telemetry history, and adoption counts repos producing receipts, not stars.

Profile settings for personal, team and enterprise use are in the [framework](#the-core-everything-is-built-from).

## A daily habit and an open format

### Do

- One command, no account, and a first result from your own sessions and bills in under a minute. Nothing leaves the machine.
- Built into the daily loop: the done gate, the status line, the Monday brief and a PR comment.
- Publish the verdict names, the receipt format and the guard list, so other tools can produce and check them without our service.
- Lead with the pain: "stop asking if it's done". Post your own numbers.
- Count weekly active repos producing verdicts or receipts, not stars.
- Keep the core small and take outside work as extensions. As pi does, close outside PRs to the core automatically and review them in a daily batch.

### Don't

- Become another spend tracker or dashboard.
- Build leaderboards. They get gamed.
- Write hooks into a repo's settings, where npm worms hid malicious ones this year.
- Depend on one agent vendor or one maintainer.

## What we reuse and what we build

### Reuse, through adapters

- **Traces, errors and analytics:** OpenTelemetry, Langfuse, PostHog, Sentry.
- **Spend for the big model vendors:** free trackers from Ramp, SuperPenguin and Vantage.
- **Cloud inventories and cost diffs:** Cartography, CloudQuery or Steampipe plugins where they cover a vendor, and Infracost for Terraform changes.
- **Running evals and tests:** Inspect, promptfoo, Harbor, your own test runner.
- **Sandboxes, policy and approvals:** whatever the team already runs, including each agent's own permission tiers and hooks.
- **Rollouts:** your host's gradual deploys.
- **How agents should work:** skill packs such as pstack. They're advice; `dr` is the check behind them.

### Build, because nobody owns it

- Proof of each change, made by the tool, with honest verdicts.
- Receipts tied to what ships, plus a small verifier anyone can run.
- Checks on the checks: did each gate run, and is each meter right and fresh?
- Blocked time, review wait and rework, measured.
- Guards for failure classes across agents, with numbers on whether they work.
- One meter across every vendor, checked against billing emails.

Not built: an approval or policy engine, another model-spend tracker, trace storage, an eval runner, or leaderboards.

## Existing tools plug into the concepts

| Concept | Plugs into |
|---|---|
| Readings | Vendor billing and usage APIs, billing email, FOCUS exports, OpenTelemetry, Langfuse, PostHog, Sentry, agent session logs, git, CI history |
| Checks | Playwright, Maestro, Vitest, pytest, tester-army/e2e, Harbor, Inspect, promptfoo, k6, Lighthouse, property-testing libraries, and Qlty CLI as an optional static checker |
| Lanes | Your machine, GitHub Actions, GitLab CI or any other CI, nightly schedulers, live test accounts |
| Guards | Claude Code, Codex and Cursor hooks; managed settings; CODEOWNERS and branch protection |
| Skills | Agent Skills packs such as pstack, installed with the skills CLI; `dr` measures them like any other configuration change |
| Receipts | in-toto, SLSA, Sigstore, and any evidence ledger the team already runs |
| Decisions and open items | PR reviews, issue trackers, chat |

## What we copy from teams already doing this

Four sources shaped the details. One is Anthropic's own write-ups on making claude.ai faster, cutting cost per task, running evals and triaging CI with agents. Another is formal verification, where we borrow the discipline, not the proofs. In a 2025 benchmark, the best model could fully prove only 4.9% of tasks, so proving app code isn't practical yet. Making each check impossible to fake is. The third is how OpenClaw and hermes-agent keep their own agent-built code working; see [Lessons](#what-breaks-in-two-of-the-busiest-agent-built-projects). The fourth is Anthropic's [AI-native SDLC playbook](https://claude.com/blog/the-ai-native-sdlc-playbook), which gives every stage from plan to maintenance a gate, a committed record and a measure. Most of its testing and hook advice is already here. We adopt its measures, its configuration evals, its control bands and its rehearsed rollbacks. One thing we don't copy: it puts team hooks in the repo's settings file, and we install hooks only at user level or through managed settings, because malicious packages have hidden hooks in repo settings. We also borrow from [pstack](https://github.com/cursor/plugins/tree/main/pstack), the skill pack poteto wrote at Cursor for rigorous agent work, mirrored for any agent at [backnotprop/pstack](https://github.com/backnotprop/pstack). Its skills are advice an agent may follow. `dr` is the check behind that advice, and it measures whether a pack like this helps on your own past tasks.

| Practice | Where it comes from | What it changes | Release |
|---|---|---|---|
| Ceilings that only move down | claude.ai got 3.1x faster in two weeks with 150+ of them. A PR that raises one fails, a daily job lowers it when things improve, and agents shipped 3,000+ changes with no customer incidents. | Speed and cost budgets tighten automatically. They're measured with steady lab counts, such as CPU instructions, rather than noisy timings. | R2, R4 |
| A named owner for anything users see | The same project: every user-visible change needed its owner's approval and rolled out to staff, then 1%, then everyone. | Taste calls go to the owner of that area, which might be you or a teammate, never to whoever is around. | R2, R4 |
| Cost per task, not per token | Anthropic's cost write-up: turns, cache reads, output and model set the price of a task. A test that catches an error costs one turn; a retry resends the whole conversation. | The meter reports dollars per finished task, and compares it with a typical developer day. | R1 |
| A strict proof protocol | Their eval guide: cases from real transcripts first, a train set and a held-out set, one change per round, kept only if both improve. A support task went from 4.6¢ to 1¢ per ticket while held-out accuracy rose from 78.6% to 90.5%. | "Change with proof" follows this protocol. Graders run twice to make sure they agree with themselves. | R3 |
| Checks as skills, on every PR | Their verification skills package the checks developers keep repeating and run them on each PR. | Your definition of done ships as a skill plus a PR check, not a prompt. Prompts lose "double-check your work"; checking happens in the gate. | R2 |
| One rule per bug class | Formal methods: name the invariant once and test it with many generated inputs. A property test catches about 50 times as many planted bugs as a single-case test. | "The count on the page equals the count in the email and in history" replaces many one-off tests. Old test cases become its starting inputs. | R4 |
| Every check proves it can fail | A check must fail on the old code and catch a planted copy of the bug. Meta's engineers accepted 73% of tests built this way. | Tests that can never fail get rejected, including ones that copy their expected value from the code under test. | R2, R4 |
| Agents can't edit the checks | In one benchmark of impossible tasks, models cheated 76% of the time by editing tests or hardcoding answers. Read-only tests and a legitimate "this conflicts with the spec" exit cut it sharply. | Hooks block agents from editing checks, budgets and their stored inputs. A conflict comes to you as a taste call. | R2, R4 |
| Hand back the smallest failing case | Counterexample-guided loops: the checker returns one concrete, minimal failure, and that case is stored forever. | The gate tells the agent exactly what failed, not just "tests failed", and fixed bugs stay fixed. | R2 |
| The same rule in production | Stripe-style reconciliation: check the invariant on live data after a short delay. | A mismatch in production becomes a replayable failing case and a finding. | R2 |
| Check the merged result | When two branches touch the code behind one invariant, run it on the merge. | Team guards catch teammates silently undoing each other. | R4 |
| Fixed rules for flaky CI, then a first report fast | Anthropic's CI on-call agent: fixed rules filter flaky failures and noise, the first analysis lands in a median 14 minutes, a lessons file feeds the next one, and a human approves before merge. | The CI release copies this, with a separate limited-permission agent for deploys. | R5 |
| Memory that learns | [Hindsight](https://github.com/vectorize-io/hindsight): facts kept with exact quotes and a proof count, living knowledge pages written as markdown, and search that fuses keyword, vector, graph and time. | Wiki claims carry proof counts, and standing questions become pages the wiki rewrites as it learns. Hindsight itself can be an optional extension behind the wiki's index; it brings its own model and server, so it's never the default. | R1, later |
| The history is the memory | [OptChat](https://gist.github.com/VictorTaelin/91837951a5ce5b38f341ec1ba1df6449), an open spec: every message is kept word for word, a cheap model folds the log into a tree of one-line summaries with the user's own words ranked first, and the agent zooms into a line when it needs detail. A correction given in chat outlives the instruction file. | The ledger keeps the full history across all agents. Any agent can search it or zoom into past decisions and corrections through MCP, and the latest ruling wins. | R4 |
| A browser speed playbook | Anthropic's computer-use guide: shrink screenshots, keep only the last few at full size so the cache keeps working, batch independent actions, and use a faster model where waiting matters. | Ready-made changes the tool can propose and prove for slow browser tasks. | R3 |
| Warn until a replay proves precision | hermes-agent's code-health rules only warn until a frozen replay of past merged PRs shows they're precise, and their switch is read from main ([baa0769](https://github.com/NousResearch/hermes-agent/commit/baa07694a8e326c2b77c5f00880eaf948fd38b04)). | Guards move from warning to blocking the same way, and no change can relax its own guard. | R4 |
| Keep a test only if it fails when its fix is reverted | hermes-agent used exactly this test to restore 175 of the roughly 8,200 tests it had deleted ([#120220](https://github.com/NousResearch/hermes-agent/pull/120220)). | The bite check decides which tests stay and which go. | R3 |
| Known failures keyed to the exact error | hermes-agent's [list of pending fixes](https://github.com/NousResearch/hermes-agent/blob/main/tests/e2e/core/_pending_fixes.py) marks each expected failure by its error text. | An accepted failure can't hide a new one with a different error. | R2 |
| A bypassed boundary is not proof | OpenClaw's [agent rules](https://github.com/openclaw/openclaw/blob/main/AGENTS.md) ask for the real failing entry point first, and a [test-audit skill](https://github.com/openclaw/openclaw/blob/main/.agents/skills/test-audit/SKILL.md) gates new tests. | The plan check rejects tests that skip the boundary they claim to cover. | R1 |
| A strict bar for flaky tests | OpenClaw counts a flaky test as fixed only after 20 clean standalone runs and 3 clean runs of its original shard ([testing guide](https://github.com/openclaw/openclaw/blob/main/docs/help/testing/writing-tests.md)). | A flaky verdict needs repeated clean runs, never a rerun until green. | R1, R5 |
| Leased real test accounts | OpenClaw's Telegram tests run on real accounts leased for each run ([skill](https://github.com/openclaw/openclaw/blob/main/.agents/skills/telegram-e2e-userbot/SKILL.md)). | The environment manifest leases accounts, numbers and devices to one session at a time. | R1 |
| Releases record their evidence and waivers | OpenClaw's [release process](https://github.com/openclaw/openclaw/blob/main/docs/reference/RELEASING.md) records the commit, the evidence and any waived step, such as the skipped soak in [2026.9.5](https://github.com/openclaw/openclaw/releases/tag/v2026.9.5). | A waiver is a decision, and its outcome is measured. | R3 |
| A cooldown on new dependencies | OpenClaw takes new dependency versions only after they're seven days old ([#158298](https://github.com/openclaw/openclaw/pull/158298)). | Upstream updates wait unless a fix needs them, and are re-proved when taken. | R2 |
| Observed or modeled | hermes-agent's [post-mortem harness](https://github.com/NousResearch/hermes-agent/blob/main/evals/postmortem/README.md) labels every number as observed or modeled. | Every reading says whether it was measured or estimated. | R2 |
| Agent configuration gets regression tests | The SDLC playbook runs 20 to 50 real past tasks as evals whenever the instruction file, skills or hooks change, and on a schedule. A change that lowers the pass rate goes back for review, and every incident adds a task. | `dr` builds the task set from your own transcripts. Every change to instruction files, skills, hooks, guards or policy is replayed against it before it lands, including the rules `dr` writes itself. | R4 |
| A plan with a proof section | In the playbook, work starts from a committed plan that lists the files that change, the order of work, the risks and the proof. Later stages check the diff against it. | The done gate runs the plan's proof steps and compares the merged diff with the plan. Plan drift and first-pass merges are readings, and the files listed feed the team claims. | R1, R4 |
| A verifier with fresh eyes | The playbook's verifier runs in a new context, exercises the changed behavior and the two nearest neighboring flows, and reports without fixing anything. | The final journey check runs in a context that didn't write the code, and covers the nearest neighbors of the change as well as the change itself. | R1 |
| Control bands, not fixed thresholds | The playbook watches a metric against a rolling baseline with standard drift rules. Detection uses no model. A small breach is logged, a larger one gets a read-only diagnosis, and only the largest may open a PR or run a pre-approved runbook. | Alerts use the same bands and tiers. Dismissals with a reason tune the bands, so noisy fixed-threshold alerts stop. | R2 |
| Rollback is the most rehearsed path | The playbook exercises rollback regularly in staging so it's proven before an agent ever needs it. | An automated action that relies on a rollback nobody has rehearsed recently is BLOCKED. | R3 |
| Review findings with severity, and an author who can't approve | The playbook's review runs fixed passes (bugs, security, match with the spec and plan), caps the nits, and leaves approval to a human code owner. A finding seen twice goes into the instruction file. | Review findings become open items ranked by severity, the agent that wrote a change can't approve it, a repeated finding becomes a rule or guard, and each reviewer's precision is measured. | R4 |
| A leading and a lagging measure for every stage | Each of the playbook's plays names its measures and reads them from records teams already keep: git timestamps, PR metadata, CI and OpenTelemetry. | These come built in as readings: time from issue to merge, first-pass merges, rework cycles, time to first review, time waiting at each approval, and the DORA measures. | R1, R2 |
| One source of truth per record | The playbook names one system as the authority for each artifact, and at minimum links the tracker record and the commit both ways. | Every ledger item carries its issue ID and commit, and the verdict is linked back on the issue. | R1 |
| Fix a repeated mistake at the highest level | pstack's [correct skill](https://github.com/backnotprop/pstack/blob/main/skills/correct/SKILL.md) fixes each class of repeated mistake with structure first, then types, then a lint whose error names the fix, then a test, and docs last. It keeps a table pairing each rule with what enforces it. | The same ladder and rule table for every lesson `dr` learns, with each new check proven on a real past mistake. | R4 |
| A feature map kept honest | pstack [generates a verification skill](https://github.com/backnotprop/pstack/blob/main/skills/create-verification-skill/SKILL.md) with a map of user-facing features. A [maintenance pass](https://github.com/backnotprop/pstack/blob/main/skills/maintain-verification-skill/SKILL.md) drives every feature live and keeps drift in the map apart from real regressions. | Journeys start from a feature map, and features with no journey are a reading. | R1, R3 |
| The "returns nothing" test | pstack's [testing rule](https://github.com/backnotprop/pstack/blob/main/skills/principle-test-behavior-not-implementation/SKILL.md): a test that would still pass if every function it imports returned nothing gets rewritten or deleted. It names five common shapes of such tests. | The first and cheapest bite check. | R3 |
| Question the premise after two failed fixes | pstack's [rule](https://github.com/backnotprop/pstack/blob/main/skills/principle-attack-the-premise/SKILL.md): when two fixes that share a premise fail the same gate, write the premise down and measure it before any third fix. | Rule 2: a third patch on a failed assumption becomes a premise review in the inbox. | R4 |
| Blinded evals | pstack's [eval playbook](https://github.com/backnotprop/pstack/blob/main/skills/poteto-mode/playbooks/eval.md): candidates don't know they're being tested, a judge from another model family sees only anonymized outputs, and what each agent did is read from its transcript, not its own report. | Configuration evals are graded the same way. | R4 |
| Enforce a background verdict at the next stop | Justin Searls' [prove_it](https://github.com/searlsco/prove_it) runs expensive reviewers in the background while Claude keeps working, then enforces their verdict when it next stops. It also triggers checks on lines changed and when the agent loops. | Slow checks report at the next stop. Churn and loop triggers and reviewer subagents come with the guards. | R1, R4 |
| Local CI as a commit status | Basecamp's [gh-signoff](https://github.com/basecamp/gh-signoff) runs tests on your own machine and posts the result as a GitHub commit status. | Every verdict is posted the same way, so branch protection can require it without hosted CI. | R1 |
| No evidence, no done | [donecheck](https://github.com/AtharvaMaik/donecheck) fails when there's nothing to verify, flags placeholder text in changed files, and marks a receipt stale when its inputs change. [DoneGate](https://github.com/Tetusa1/DoneGate) requires the right commit, owned paths and a lease, not an exit code. | The done gate flags placeholders, verdicts go stale with a new commit, and team guards check path ownership. | R1, R4 |

## How the framework knows it's working

| Measure | Target |
|---|---|
| Silent checks | Zero: every check reports what it ran and its last real pass |
| Stand-in proof | Zero claims that users would notice are proven only on stand-ins |
| Actions on unchecked readings | Zero; money totals within 2% of the bills |
| Proof coverage | Most fixes carry tool-made, fail-before-fix proof tied to what ships |
| Guard precision | False-positive rate known for every guard |
| Recurrence | Guarded classes stay away for 30 days, including nearby surfaces |
| Blocked time | Visible weekly by cause and owner, with the top cause shrinking |
| Repeat questions | No decision asked twice; "is it done?" follow-ups down by half |

## When each part lands

| Release | Concepts | Rules |
|---|---|---|
| R1 Core + prove "done" | The core's six: Event, Claim, Check, Verdict, Ask, Decision | 1, 3, 4 |
| R2 Find + trust the numbers | Spec budgets, Trigger | 5, 8, 9 |
| R3 Change with proof | Receipt, Lane, Spec environment | 1, 10 |
| R4 Keep | Guard, failure classes, Owner | 2, 6, 7 |
| R5 CI | CI lanes and policies | 8 |
| R6 Run your workflow | Playbooks made of Claims, Checks and Decisions; quota Readings per account | 9, 10 |

The release details are in the [roadmap](#proof-of-done-first-then-everything-else-builds-on-it) above.

## Product decisions to make

| Decision | Recommendation | Needed by |
|---|---|---|
| Name | DoneRight, chosen on October 5 because it says the job in plain words. It replaced the working name Flight Recorder, which read as aviation or Java. | Done |
| Core size | The core is the record, gates, asks and adapters. Everything else ships as an extension, including our own features, and a new core feature needs a reason no extension can meet. | R1 |
| Extension trust | Extensions run as code. They install at user level and are pinned to a version. A project's extensions load only after you trust the project. | R1 |
| Language | TypeScript on Node for the core, extensions, MCP server and team server; Rust for the hook client and the app. Extensions load in-process as TypeScript modules, the way pi's do, and everything around the product (MCP, Playwright, agent plugins, Cloudflare Workers) is TypeScript too. The hook client is a tiny Rust binary that forwards each hook call to the running hub and allows the action if the hub is down, because hooks run on every tool call. Rust comes in elsewhere only where profiling shows a need, as a native module. Contracts are JSON Schemas with conformance tests and a JSON-RPC mode, so extensions in any language can run out of process. Effect 4 is decided by a one-day spike before the hub is built: if it passes, it's used only inside the hub core, and the CLI, the extension API, contracts and storage stay plain TypeScript. | R1 |
| Receipt format | Build on an existing attestation standard (in-toto or SLSA) so CI and deploy tools can verify it | R3 |
| Local app | One native desktop app, built with gpui-kit (Rust on GPUI, Apache-2.0): 75+ components, accessibility through AccessKit, headless UI tests, a webview, and gpui-shell, a JavaScript extension host where every capability is granted explicitly. It's native on macOS, and the same app runs on Windows and Linux later. There is no web view; HTML is only for reports you share, and the app shows them in its webview. R1 brings the window with its three views and its box for changes in words, the count in the menu bar, notifications with answers and the hub at login; it sets itself up on first launch, with no setup screen or Settings page. R2 adds the line under the menu bar, modeled on tendedero, in the same app. R6 adds the Flow view, with playbooks. A cursor helper like Clicky is an Add-anything extension, not another app. A two-day spike comes first. If gpui-kit fails it, we fall back to the earlier plan: a small Swift app for the line, plus a local web view. | R1 |
| Hosted app | Not in v1. Terminal, the app and report files first. The team server is a push-only sync, content-free by default: self-hosted with Docker, or on Cloudflare for a small team, and in the customer's own cloud for an enterprise. Teammates use the same app, pointed at it. Only after Decision 2. | Decision 2 |
| FinOps scope | Infra cost for the vendors you use in R2 and R3. Billing emails, every other vendor and commitments come after Decision 2. | R2 |
| License | Apache-2.0; decide later whether a paid enterprise or team version is worth building | R1 |
| First agents | Claude Code and Codex in R1; pi and Cursor in R2 | R1 |
| Policy limits | Policy answers only reversible, in-scope questions on its own. Anything irreversible, paid or outward-facing still asks you unless an explicit rule covers it. | R1 |
| Gate mode | Before merge by default. For teams that push to main all day, after merge, with "done" judged at release. | R2 |
| Live test budget | A monthly cap per real resource that you set, plus budgets you approve once per issue: runs, a dollar cap and an end date. Runs inside a budget never ask; otherwise live journeys run only when asked or before a release. | R1 |
| Where configuration evals run | On your machine, or in a lane you dispatch with a budget. Never on every push in hosted CI, since each run makes paid model calls. | R4 |
| Memory | One local wiki, compiled from your history and each agent's memory, every line with its source. Agents read it and propose changes; they never write it. Search over it is an extension, so its index can be swapped without losing anything. | R1 wiki, R4 memory |
| Subscriptions | Used only through the unmodified Claude Code and Codex on your own sign-in. DoneRight never holds credentials or routes requests through a plan. Hooks, held stops and sessions you asked for are ordinary use; each scheduled automation it starts on its own is capped or moved to an API key. | Done |
| Several accounts | Account-agnostic. DoneRight never signs in, switches accounts or suggests a switch. It tracks each account's windows separately, so a newly signed-in account isn't mistaken for a reset, and queued work carries on with whichever account you sign in to. | Done |
| Enterprise design partner | One large organization, run under its own confidentiality rules, with its data and evidence kept inside it | Before the enterprise pilot |

## Where this comes from

- [How claude.ai got 3x faster in two weeks](https://claude.dev/blog/how-we-made-claude-ai-faster/)
- [What a task costs on Opus 5.5](https://claude.dev/blog/what-a-task-costs-on-opus-5-5/)
- [Automating eval design and hillclimbing](https://claude.dev/blog/automating-eval-design-and-hillclimbing/)
- [Verification loops in Claude Code with skills](https://claude.com/blog/building-verification-loops-in-claude-code-with-skills)
- [Claude as CI/CD first responder](https://claude.com/blog/ai-ci-cd-on-call)
- [Computer and browser use best practices](https://claude.com/blog/best-practices-for-computer-and-browser-use-with-claude)
- [Kleppmann on AI and formal verification](https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html)
- [ImpossibleBench: agents gaming tests](https://arxiv.org/abs/2510.20270)
- [Meta: tests that catch planted bugs](https://arxiv.org/abs/2501.12862)
- [The AI-native SDLC playbook](https://claude.com/blog/the-ai-native-sdlc-playbook)
- [OpenClaw's rules for agents](https://github.com/openclaw/openclaw/blob/main/AGENTS.md)
- [hermes-agent's testing rules for agents](https://github.com/NousResearch/hermes-agent/blob/main/tests/AGENTS.md)

---

Product, framework and roadmap in one document. Built from research run October 3–5, 2026: cost tooling across vendors, FinOps for the agent era, evaluation, CI, test formats, formal verification, published engineering practice, a six-month cross-check of public discussion, adoption of similar tools, three weeks of commits matched to issues in OpenClaw and hermes-agent, and a 30-day pilot on one founder's own data. The examples are illustrative.
