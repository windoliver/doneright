# Flight Recorder

Measure and prove every change your coding agents make.

Writing code is cheap now. Knowing a change is right, and what it cost, is the bottleneck. Flight Recorder turns an agent's claim ("done", "fixed", "no visible change") into a verdict without you, using evidence the tool makes itself. It brings you only the calls that need a person.

> **Status: design.** Nothing is built yet. The plan lives in the [issues](https://github.com/windoliver/flight-recorder/issues) and [milestones](https://github.com/windoliver/flight-recorder/milestones). "Flight Recorder" is a working name.

## How it works

The core has four parts. Everything else is an extension you can turn off, replace or write yourself, the way [pi](https://github.com/earendil-works/pi) keeps its core small.

| Part | What it does |
|---|---|
| **Record** | An append-only event log plus a folder of evidence, named by content hash, on your machine |
| **Gates** | When a trigger fires (an agent says it's done, a command is about to run, a PR opens), a gate runs the checks for a claim and returns a verdict |
| **Asks** | The only thing a person ever sees. Saved decisions answer repeat questions first, and your answer goes back into the agent's session. |
| **Adapters** | One hook contract for Claude Code, Codex, pi and later Cursor |

Every verdict is one of five:

| Verdict | Means | Next step |
|---|---|---|
| `PASS` | The check failed before the change and passed after, over enough runs | Nothing |
| `FAIL` | Something got worse | The agent gets the smallest failing case back |
| `INCONCLUSIVE` | Too few runs to tell | More runs are scheduled |
| `BLOCKED` | Something outside the code is missing: an environment, access or an approval | Its owner gets the request |
| `INVALID` | The check is broken, can't fail, or ran nothing | Fix the check |

An item reaches you only for **taste** (a judgment only you can make), **authority** (anything irreversible, outward-facing or over budget), **unblock** (something only you can provide), or an **exception** (something the system can't settle). The whole record stays visible in `fr view` whenever you look, but nothing else is pushed.

## Principles

- **Local first.** It runs and stores everything on your machine, and sends nothing anywhere by default.
- **No SDK.** It reads what agents, apps and vendors already record: transcripts, hooks, OpenTelemetry, your app's own tables and vendor usage APIs.
- **Agents can't grade their own work.** An agent's word is never evidence, and agents can't edit checks or decisions.
- **Missing is never passing.** A skipped or unrun check is `BLOCKED` or `INVALID`, never green.

## Docs

- [Product](docs/product.md): the problem, the product, what it solves, the roadmap and the decisions.
- [Technical design](docs/technical-design.md): architecture, data model, interfaces, security and the build plan for version 0.

## Layout (planned)

```
packages/   core, hub, cli, view, mcp, contracts, first-party extensions, adapters  (TypeScript, Node 22.19+)
crates/     fr-hook, the tiny hook client every agent hook calls                   (Rust)
docs/       product and technical design
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). The core stays small, and extensions are where most work goes.

## License

[Apache-2.0](LICENSE)
