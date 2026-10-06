# Contributing

DoneRight is in its design phase. The [issues](https://github.com/windoliver/doneright/issues) are the plan: each epic is a release step, and each task lists its acceptance criteria, the proof it needs, and what it depends on.

## Where work goes

- **The core** is the record, gates, asks and adapters. It stays small. Open an issue before changing it. Pull requests to the core that weren't agreed in an issue may be closed and reviewed in a batch, the way [pi](https://github.com/earendil-works/pi) handles outside changes to its core.
- **Extensions** are where most features live: checks, resolvers, sources, environments, views and adapters. New extensions are welcome.

## Every pull request carries proof

- Link the issue with `Closes #n` only when every acceptance criterion is proven. Otherwise use `Part of #n`.
- Show the check failing before your change and passing after: the command and its output, or a link to the evidence.
- Add screenshots or a video for anything a user sees.
- List anything still open.

## Agents

Agent-written contributions are welcome. Say which agent and model you used. You're responsible for the change, and [AGENTS.md](AGENTS.md) applies to you and your agent.

## License

By contributing, you agree that your contributions are licensed under the [Apache License 2.0](LICENSE).
