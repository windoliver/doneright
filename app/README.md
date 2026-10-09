# DoneRight app prototype

A native prototype of the DoneRight app, built with [gpui-kit](https://github.com/longbridge/gpui-kit). It is the first step of the toolkit spike in [#60](https://github.com/windoliver/doneright/issues/60). The data is made up, and nothing is wired to a real hub yet.

What it shows:

- **First launch:** one **Set up** button. Setup runs on its own (reads your history into the wiki, adds watch-only hooks), then opens the map.
- **Flow**, the home screen, with three maps you switch between:
  - *Fix #123*, animated: the issue moves through the agent, its claim and the checks, is sent back once, passes on round 2, and stops at you for a taste call you answer in the right-hand panel.
  - *Start an app*: a playbook with research, your plan approval, three lanes in parallel, the prove hub, merge and your release.
  - *Add anything*: an extension built by your own agent, checked, approved, and improving with use.
- **Needs you:** every ask in one place (taste call, a memory decision, an unblock, a budget approval, new access), answered in place.
- **Sessions:** every agent in every repo, with flags (same issue, interrupted, not pushed) and Open, Pause and Note on each row.
- **Map:** each part of a repo with its sessions, issues, evidence and gaps.
- **Numbers:** plan windows with your reserve, this week's counts, and infra.
- **Add anything:** a working input box, builds that finish and install, installed extensions, and two memories compared side by side.
- **Learned:** what it learned, with where each change applies and a way to roll it back.
- **Wiki:** what it knows, every line with its source, and the questions it holds until they matter.

## Run it

Needs Rust and Xcode (for Metal) on macOS 14 or later. The first build compiles GPUI and takes several minutes.

```sh
cd app
./scripts/bundle-macos.sh
open DoneRight.app
```

For checks and screenshots you can open a given screen directly:

```sh
DR_START=flow DR_T=29 DR_PAUSED=1 ./DoneRight.app/Contents/MacOS/DoneRight
```

`DR_START` takes `setup`, `flow`, `needs`, `sessions`, `map`, `add`, `learned`, `wiki` or `numbers`. `DR_FLOW` picks the flow map (0 fix, 1 playbook, 2 extension). `DR_T` is seconds into the fix animation (17.5 is the sent-back moment, 29 the taste call), `DR_PAUSED=1` holds it still, and `DR_BUILD=1` starts Add anything with a build ready to install.
