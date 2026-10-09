# DoneRight app prototype

A native prototype of the DoneRight app, built with [gpui-kit](https://github.com/longbridge/gpui-kit). It is the first step of the toolkit spike in [#60](https://github.com/windoliver/doneright/issues/60). The data is made up, and nothing is wired to a real hub yet.

What it shows:

- First launch: one **Set up** button. Setup runs on its own (reads your history into the wiki, adds watch-only hooks), then opens the map.
- **Flow**, the home screen: a live map of the work. Issue #123 moves through the agent, its claim and the checks, is sent back once, passes on round 2, and stops at you for a taste call. Answer it in the right-hand panel.
- **Needs you**, **Sessions** and **Learned**, in the sidebar.

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

`DR_START` takes `setup`, `flow`, `needs`, `work` or `learned`. `DR_T` is seconds into the Flow animation (17.5 is the sent-back moment, 29 the taste call), and `DR_PAUSED=1` holds it still.
