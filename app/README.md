# DoneRight app prototype

A native prototype of the DoneRight app, built with [gpui-kit](https://github.com/longbridge/gpui-kit). It is the first step of the toolkit spike in [#60](https://github.com/windoliver/doneright/issues/60). The data is made up, and nothing is wired to a real hub yet.

What it shows:

- **First launch:** one **Set up** button. Setup runs on its own (reads your history into the wiki, adds watch-only hooks), then opens the map.
- **Flow**, the home screen: the gadgets your agents added, then one of four maps:
  - *Fix #123*, animated: the issue moves through the agent, its claim and the checks, is sent back once, passes on round 2, and stops at you for a taste call you answer in the right-hand panel.
  - *Start an app*: a playbook with research, your plan approval, three lanes in parallel, the prove hub, merge and your release.
  - *Add anything*: an extension built by your own agent, checked, approved, and improving with use.
  - *Suggested*: DoneRight noticed you asking "did it deploy?" nine times, your agent built a deploy gadget in a copy, its checks passed, and it waits for one tap.
- **The app extends itself.** Whatever your agents build shows up in the app: an item in the sidebar under *Added by you*, a gadget on the home screen, or a tab inside a claim (the *Receipt* tab, which DoneRight suggested from how you work). **Show what was added** in the title bar outlines every added piece and names the sentence it was built from. Click an outline to see where it came from: the agent, what it cost of your plan, its checks, what it can and can't do, its history, and Roll back or Remove.
- **Suggestions** come one at a time and never count as asks. One waits on the home screen, in Needs you and in Add anything, with a notification the first time.
- **⌘K, or Tell DoneRight…** in the sidebar: type a sentence and pick it to have your agent build it, or jump to any screen.
- **Needs you:** every ask in one place (taste call, a memory decision, an unblock, a budget approval, new access), answered in place.
- **Sessions:** every agent in every repo, with flags (same issue, interrupted, not pushed) and Open, Pause and Note on each row.
- **Map:** each part of a repo with its sessions, issues, evidence and gaps.
- **Numbers:** plan windows with your reserve, this week's counts, what was sent back before it reached you, and infra.
- **Add anything:** a working input box, builds that finish and add themselves to the app, everything added so far, and two memories compared side by side.
- **Learned:** what it learned, with where each change applies and a way to roll it back.
- **Wiki:** what it knows, every line with its source, and the questions it holds until they matter.

Built from gpui-kit's components: TitleBar, Sidebar, TabBar, Stepper, DescriptionList, Notification, Sheet, Dialog with Command, Switch, Tooltip, Kbd, ShimmerText, BarChart, Tag, Progress, Input and Button. The maps are drawn on GPUI's canvas.

In the real app, an added piece is a script the app loads through gpui-shell, gpui-kit's extension host: no rebuild, the same components as the rest of the app, and no access beyond what its grant declares. Here the added pieces are Rust stand-ins.

## Run it

Needs Rust and Xcode (for Metal) on macOS 14 or later. The first build compiles GPUI and takes several minutes.

```sh
cd app
./scripts/bundle-macos.sh
open DoneRight.app
```

## Open a state directly

```sh
DR_START=flow DR_T=29 DR_PAUSED=1 ./DoneRight.app/Contents/MacOS/DoneRight
```

- `DR_START`: `setup`, `flow`, `needs`, `sessions`, `map`, `add`, `learned`, `wiki`, `numbers`, or `ext:N` for added piece N's panel.
- `DR_FLOW`: the flow map (0 fix, 1 playbook, 2 extension, 3 suggested).
- `DR_T`: seconds into the fix animation (17.5 is the sent-back moment, 29 the taste call). `DR_PAUSED=1` holds it still.
- `DR_ADDED=1`: start with Show what was added on. `DR_DTAB=1`: the claim's Receipt tab.
- `DR_BUILD=1`: a build ready to add. `DR_BUILD=installed`: one already added.
- `DR_PROPOSAL=accepted` or `dismissed`: the suggestion already answered. `DR_QUIET=1`: no suggestion notification.
- `DR_TOAST=propose`, `accepted`, `installed` or `answer`: show that notification at start. `DR_HOLD=1` keeps notifications up.
- `DR_SHEET=N`: open where added piece N came from. `DR_PALETTE="text"`: open ⌘K with text typed.

## Headless snapshots

Every screen can be rendered without a display, with GPUI's Metal test renderer, so the screenshots also work with the screen locked. States that follow a click are reached by real clicks, keys and typing in that renderer, which checks the handlers too.

```sh
python3 scripts/snapshots.py
```

It builds the `snap` variant into `target-snap/` and writes 22 PNGs to `snapshots/`. To render one state yourself:

```sh
DR_SNAP=out.png DR_START=flow DR_T=17.5 DR_PAUSED=1 DR_DO="click:show-added; click:mark#0" target-snap/debug/doneright-app
```

`DR_DO` steps are separated by `;`: `click:ID`, `hover:ID`, `press:KEYS`, `input:TEXT` and `wait:MS`, where `name#3` is the element id `("name", 3)`. The renders have no window chrome, so the snap variant draws the window controls in.
