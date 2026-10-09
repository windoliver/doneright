//! DoneRight app prototype on gpui-kit.
//! First launch shows one button. Setup runs after the click, with nothing else to do,
//! and the home screen is a live map of the work: a task moves through the agent, its
//! claim, the checks and you, with the sent-back loop drawn where it happens.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

mod ext;
mod more;
#[cfg(feature = "snap")]
mod snap;
use ext::*;
use more::*;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::{
    ActiveTheme, Disableable as _, Icon, Selectable as _, Sizable as _, StyledExt as _, Theme, ThemeMode, TitleBar, h_flex, v_flex,
    button::{Button, ButtonVariants as _},
    command::CommandState,
    description_list::DescriptionList,
    input::{Input, InputEvent, InputState},
    kbd::Kbd,
    chart::BarChart,
    progress::Progress,
    shimmer::ShimmerText,
    sidebar::{Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
    stepper::{Stepper, StepperItem},
    switch::Switch,
    tab::{Tab, TabBar},
    tag::Tag,
    tooltip::Tooltip,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

// ---------- the four status colors, the same everywhere ----------
fn c_run() -> Hsla { hsla(248. / 360., 0.70, 0.60, 1.) }
fn c_back() -> Hsla { hsla(22. / 360., 0.78, 0.52, 1.) }
fn c_ok() -> Hsla { hsla(145. / 360., 0.55, 0.40, 1.) }
fn c_you() -> Hsla { hsla(330. / 360., 0.67, 0.48, 1.) }

#[derive(Clone, Copy, PartialEq, Default)]
enum St { #[default] Idle, Run, Ok, Back, You }

#[derive(Clone, Copy, PartialEq)]
enum Screen { Welcome, Setup, App }

#[derive(Clone, Copy, PartialEq)]
enum View { Flow, NeedsYou, Sessions, Map, Add, Learned, Wiki, Numbers, Ext(usize) }

actions!(doneright, [OpenPalette]);

// ---------- the map: stations on one line, the checks as spokes on a hub ----------
const W: f32 = 860.;
const H: f32 = 300.;
const Y: f32 = 120.;
const X_TICKET: f32 = 46.;
const X_AGENT: f32 = 176.;
const X_CLAIM: f32 = 304.;
const X_CHECKS: f32 = 452.;
const X_YOU: f32 = 612.;
const X_PR: f32 = 724.;
const X_MERGED: f32 = 818.;
const HUB_R: f32 = 24.;
const SPOKE_R: f32 = 96.;
const CHECKS: [&str; 6] = ["unit", "types", "merge base", "journey ×29", "screens", "size"];

/// One pass of issue #123, in seconds at 1x.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
enum Phase {
    #[default] Ticket, ToAgent, Agent1, ToClaim1, Claim1, ToChecks1, Checks1, Back, Agent2, ToClaim2,
    Claim2, ToChecks2, Checks2, ToYou, Waiting, ToPr, Pr, ToMerged, Merged,
}
const TIMELINE: [(Phase, f32); 15] = [
    (Phase::Ticket, 1.6), (Phase::ToAgent, 1.0), (Phase::Agent1, 4.0), (Phase::ToClaim1, 0.8),
    (Phase::Claim1, 1.0), (Phase::ToChecks1, 0.8), (Phase::Checks1, 5.2), (Phase::Back, 2.0),
    (Phase::Agent2, 3.2), (Phase::ToClaim2, 0.8), (Phase::Claim2, 0.8), (Phase::ToChecks2, 0.8),
    (Phase::Checks2, 5.4), (Phase::ToYou, 0.9), (Phase::Waiting, f32::INFINITY),
];
const AFTER: [(Phase, f32); 4] = [(Phase::ToPr, 0.9), (Phase::Pr, 1.2), (Phase::ToMerged, 0.9), (Phase::Merged, 6.0)];
const ORDER: [Phase; 19] = [
    Phase::Ticket, Phase::ToAgent, Phase::Agent1, Phase::ToClaim1, Phase::Claim1, Phase::ToChecks1, Phase::Checks1,
    Phase::Back, Phase::Agent2, Phase::ToClaim2, Phase::Claim2, Phase::ToChecks2, Phase::Checks2, Phase::ToYou,
    Phase::Waiting, Phase::ToPr, Phase::Pr, Phase::ToMerged, Phase::Merged,
];

fn phase_at(t: f32, answered_at: Option<f32>) -> (Phase, f32, f32) {
    if let Some(a) = answered_at {
        let mut s = a;
        for (p, d) in AFTER {
            if t < s + d { return (p, t - s, d); }
            s += d;
        }
        return (Phase::Merged, 6.0, 6.0);
    }
    let mut s = 0.;
    for (p, d) in TIMELINE {
        if t < s + d { return (p, t - s, d); }
        s += d;
    }
    (Phase::Waiting, 0., f32::INFINITY)
}
fn phase_start(target: Phase) -> f32 {
    let mut s = 0.;
    for (p, d) in TIMELINE {
        if p == target { return s; }
        s += d;
    }
    s
}
/// The sent-back loop, from the top of the checks hub back to the agent.
fn arc_point(u: f32) -> (f32, f32) {
    let (x0, y0) = (X_CHECKS, Y - HUB_R);
    let (x1, y1) = (X_CHECKS, Y - 104.);
    let (x2, y2) = (X_AGENT, Y - 104.);
    let (x3, y3) = (X_AGENT, Y - 9.);
    let v = 1. - u;
    (
        v * v * v * x0 + 3. * v * v * u * x1 + 3. * v * u * u * x2 + u * u * u * x3,
        v * v * v * y0 + 3. * v * v * u * y1 + 3. * v * u * u * y2 + u * u * u * y3,
    )
}
fn spoke_pos(i: usize) -> (f32, f32) {
    let a = (150. - 24. * i as f32).to_radians();
    (X_CHECKS + SPOKE_R * a.cos(), Y + SPOKE_R * a.sin())
}

#[derive(Clone, Default)]
struct Snap {
    phase: Phase,
    round: u8,
    stations: [St; 7],
    seg: [St; 6],
    arc: St,
    arc_label: &'static str,
    spokes: [St; 6],
    token: (f32, f32),
    dash: f32,
    pulse: f32,
}

#[derive(Clone, Copy)]
struct Pal { bg: Hsla, fg: Hsla, muted: Hsla, muted_fg: Hsla, border: Hsla, card: Hsla, primary: Hsla, primary_fg: Hsla }
impl Pal {
    fn of(cx: &App) -> Self {
        let t = cx.theme();
        Self { bg: t.background, fg: t.foreground, muted: t.muted, muted_fg: t.muted_foreground, border: t.border, card: t.popover, primary: t.primary, primary_fg: t.primary_foreground }
    }
    fn st(&self, st: St) -> Hsla {
        match st { St::Idle => self.border, St::Run => c_run(), St::Ok => c_ok(), St::Back => c_back(), St::You => c_you() }
    }
}

struct DoneRight {
    screen: Screen,
    view: View,
    setup_t: f32,
    t: f32,
    playing: bool,
    speed: f32,
    answered_at: Option<f32>,
    answer: Option<&'static str>,
    dark: bool,
    last: Instant,
    flow_sel: usize,
    paused: HashSet<&'static str>,
    noted: HashSet<&'static str>,
    map_repo: usize,
    answers: HashMap<&'static str, &'static str>,
    input: Entity<InputState>,
    builds: Vec<Build>,
    memory: Option<&'static str>,
    rolled_back_items: HashSet<&'static str>,
    rolled_since: bool,
    // what your agents added to the app, and the one suggestion waiting
    added: Vec<Added>,
    show_added: bool,
    proposal: Proposal,
    proposal_toasted: bool,
    app_t: f32,
    detail_tab: usize,
    palette: Entity<CommandState>,
    focus: FocusHandle,
    startup: Vec<String>,
    _subs: Vec<Subscription>,
}

#[derive(Clone)]
struct Build {
    words: String,
    t: f32,
    installed: bool,
    added: Option<usize>,
}

impl DoneRight {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Tell DoneRight what you want, e.g. “hang every screenshot on a line under the menu bar”"));
        let sub = cx.subscribe_in(&input, window, |this, _, ev: &InputEvent, window, cx| {
            if let InputEvent::PressEnter { .. } = ev {
                this.submit(window, cx);
            }
        });
        cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(33)).await;
                if this.upgrade().is_none() {
                    break;
                }
                let _ = this.update_in(cx, |this, window, cx| this.tick(window, cx));
            }
        })
        .detach();
        let env = |k: &str| std::env::var(k).unwrap_or_default();
        let start = env("DR_START");
        let (screen, view) = match start.as_str() {
            "setup" => (Screen::Setup, View::Flow),
            "flow" => (Screen::App, View::Flow),
            "needs" => (Screen::App, View::NeedsYou),
            "work" | "sessions" => (Screen::App, View::Sessions),
            "map" => (Screen::App, View::Map),
            "add" => (Screen::App, View::Add),
            "wiki" => (Screen::App, View::Wiki),
            "numbers" => (Screen::App, View::Numbers),
            "learned" => (Screen::App, View::Learned),
            s if s.starts_with("ext:") => (Screen::App, View::Ext(s[4..].parse().unwrap_or(0))),
            _ => (Screen::Welcome, View::Flow),
        };
        let t0 = std::env::var("DR_T").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.);
        let mut added = seed_added();
        let proposal = match env("DR_PROPOSAL").as_str() {
            "accepted" => {
                added.push(Added { age: 1e6, ..deploy_gadget() });
                Proposal::Accepted
            }
            "dismissed" => Proposal::Dismissed,
            _ => Proposal::Pending,
        };
        let mut builds = vec![];
        match env("DR_BUILD").as_str() {
            "" => {}
            "installed" => {
                let words = "Watch the receipts inbox for bounces";
                added.push(Added { age: 1e6, ..added_from_build(words) });
                builds.push(Build { words: words.into(), t: 7., installed: true, added: Some(added.len() - 1) });
            }
            _ => builds.push(Build { words: "Watch the receipts inbox for bounces".into(), t: 6.5, installed: false, added: None }),
        }
        let mut startup = vec![];
        if !env("DR_TOAST").is_empty() {
            startup.push(format!("toast:{}", env("DR_TOAST")));
        }
        if !env("DR_SHEET").is_empty() {
            startup.push(format!("sheet:{}", env("DR_SHEET")));
        }
        if std::env::var("DR_PALETTE").is_ok() {
            startup.push(format!("palette:{}", env("DR_PALETTE")));
        }
        let palette = cx.new(|cx| CommandState::new(window, cx));
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        Self {
            screen,
            view,
            setup_t: std::env::var("DR_SETUP_T").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.),
            t: t0,
            playing: std::env::var("DR_PAUSED").is_err(),
            speed: 1.,
            answered_at: None,
            answer: None,
            dark: false,
            last: Instant::now(),
            flow_sel: std::env::var("DR_FLOW").ok().and_then(|v| v.parse().ok()).unwrap_or(0),
            paused: HashSet::new(),
            noted: HashSet::new(),
            map_repo: 0,
            answers: HashMap::new(),
            input,
            builds,
            memory: None,
            rolled_back_items: HashSet::new(),
            rolled_since: false,
            added,
            show_added: std::env::var("DR_ADDED").is_ok(),
            proposal,
            proposal_toasted: std::env::var("DR_QUIET").is_ok(),
            app_t: 0.,
            detail_tab: env("DR_DTAB").parse().unwrap_or(0),
            palette,
            focus,
            startup,
            _subs: vec![sub],
        }
    }

    fn tick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32().min(0.1);
        self.last = now;
        for job in std::mem::take(&mut self.startup) {
            let (kind, arg) = job.split_once(':').unwrap_or((job.as_str(), ""));
            match (kind, arg) {
                ("toast", "propose") => self.propose_toast(window, cx),
                ("toast", "accepted") => toast(window, cx, "Added to your home", "Deploys is on your home screen. Claude Code built it on your Max plan: 1% of one window."),
                ("toast", "installed") => toast(window, cx, "Added to Work", "“Bounces” sits in Work, next to Sessions. Built in one shot from your words; remove it any time."),
                ("toast", "answer") => toast(window, cx, "Sent to the Claude app", "“Looks right.” It carries on with #123."),
                ("sheet", i) => {
                    let i = i.parse().unwrap_or(0).min(self.added.len() - 1);
                    self.open_about(i, window, cx);
                }
                ("palette", q) => self.open_palette(Some(q), window, cx),
                _ => {}
            }
        }
        for a in self.added.iter_mut() {
            a.age += dt;
        }
        match self.screen {
            Screen::Welcome => return,
            Screen::Setup => {
                if self.playing { self.setup_t += dt; }
                if self.setup_t > 6.6 {
                    self.screen = Screen::App;
                    self.t = 0.;
                }
            }
            Screen::App => {
                self.app_t += dt;
                if self.app_t > 12. && self.proposal == Proposal::Pending && !self.proposal_toasted {
                    self.proposal_toasted = true;
                    self.propose_toast(window, cx);
                }
                for b in self.builds.iter_mut() {
                    b.t = (b.t + dt).min(7.);
                }
                if self.playing {
                    self.t += dt * self.speed;
                    if let (Phase::Merged, done, _) = phase_at(self.t, self.answered_at) {
                        if done >= 5.9 {
                            self.restart();
                        }
                    }
                }
            }
        }
        cx.notify();
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).value().to_string();
        if !text.trim().is_empty() {
            self.start_build(text.trim().to_string());
            self.input.update(cx, |s, cx| s.clean(window, cx));
        }
        cx.notify();
    }

    fn start_build(&mut self, words: String) {
        self.builds.insert(0, Build { words, t: 0., installed: false, added: None });
        self.view = View::Add;
    }

    fn restart(&mut self) {
        self.t = 0.;
        self.answered_at = None;
        self.answer = None;
    }

    fn jump(&mut self, stage: usize) {
        let targets = [Phase::Ticket, Phase::Agent1, Phase::Claim1, Phase::Checks1, Phase::ToYou, Phase::ToPr];
        let target = targets[stage.min(5)];
        if target == Phase::ToPr {
            let w = phase_start(Phase::Waiting) + 0.2;
            self.t = w;
            self.answered_at = Some(w);
            self.answer = Some("Looks right");
        } else {
            self.restart();
            self.t = phase_start(target);
        }
        self.playing = true;
    }

    fn answer_taste(&mut self, a: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        if self.answered_at.is_none() {
            self.answered_at = Some(self.t);
            self.answer = Some(a);
            toast(window, cx, "Sent to the Claude app", format!("“{a}.” It carries on with #123."));
        }
        cx.notify();
    }

    fn waiting(&self) -> bool { phase_at(self.t, self.answered_at).0 == Phase::Waiting }
    fn needs_you(&self) -> usize {
        let taste = usize::from(self.screen == Screen::App && self.waiting());
        let asks = 3usize.saturating_sub(self.answers.len());
        let memory = usize::from(self.memory.is_none());
        let builds = self.builds.iter().filter(|b| b.t >= 6. && !b.installed).count();
        taste + asks + memory + builds
    }

    fn snapshot(&self) -> Snap {
        use Phase::*;
        let (ph, el, dur) = phase_at(self.t, self.answered_at);
        let f = if dur.is_finite() && dur > 0. { (el / dur).clamp(0., 1.) } else { 0. };
        let idx = ORDER.iter().position(|p| *p == ph).unwrap_or(0);
        let past = |p: Phase| ORDER.iter().position(|x| *x == p).unwrap_or(0) < idx;
        let second = idx >= ORDER.iter().position(|p| *p == Agent2).unwrap();
        let after_back = idx >= ORDER.iter().position(|p| *p == Back).unwrap();
        let mut s = Snap { phase: ph, round: if second { 2 } else { 1 }, ..Default::default() };

        // stations: ticket, agent, claim, checks, you, pr, merged
        s.stations[0] = St::Ok;
        s.stations[1] = match ph { Agent1 | Agent2 => St::Run, Ticket | ToAgent => St::Idle, _ => St::Ok };
        s.stations[2] = match ph { Claim1 | Claim2 => St::Run, Ticket | ToAgent | Agent1 | ToClaim1 | Back | Agent2 | ToClaim2 => St::Idle, _ => St::Ok };
        s.stations[3] = match ph { Checks1 | Checks2 => St::Run, Back | Agent2 | ToClaim2 | Claim2 | ToChecks2 => St::Back, ToYou | Waiting | ToPr | Pr | ToMerged | Merged => St::Ok, _ => St::Idle };
        s.stations[4] = match ph { Waiting => St::You, ToPr | Pr | ToMerged | Merged => St::Ok, _ => St::Idle };
        s.stations[5] = match ph { Pr => St::Run, ToMerged | Merged => St::Ok, _ => St::Idle };
        s.stations[6] = if ph == Merged { St::Ok } else { St::Idle };

        // the line between stations: travelled, travelling, or not yet
        let legs = [(ToAgent, ToAgent), (ToClaim1, ToClaim2), (ToChecks1, ToChecks2), (ToYou, ToYou), (ToPr, ToPr), (ToMerged, ToMerged)];
        for (i, (a, b)) in legs.iter().enumerate() {
            s.seg[i] = if ph == *a || ph == *b { St::Run } else if past(*b) || (past(*a) && !matches!(ph, Back | Agent2) ) { St::Ok } else { St::Idle };
        }
        if matches!(ph, Back | Agent2 | ToClaim2) { s.seg[1] = if ph == ToClaim2 { St::Run } else { St::Idle }; s.seg[2] = St::Idle; }
        if matches!(ph, Claim2 | ToChecks2) { s.seg[2] = if ph == ToChecks2 { St::Run } else { St::Idle }; }

        // the sent-back loop stays drawn once it has happened
        s.arc = if after_back { St::Back } else { St::Idle };
        s.arc_label = if !after_back { "" } else if matches!(ph, Back | Agent2 | ToClaim2 | Claim2 | ToChecks2 | Checks2) { "sent back · round 1 of 3" } else { "passed on round 2" };

        // the checks, one spoke each
        let mut sp = [St::Idle; 6];
        match ph {
            Checks1 => {
                let x = f * 5.2;
                sp[0] = if x > 1.0 { St::Ok } else { St::Run };
                sp[1] = if x > 1.8 { St::Ok } else if x > 0.3 { St::Run } else { St::Idle };
                sp[2] = if x > 2.6 { St::Ok } else if x > 0.6 { St::Run } else { St::Idle };
                sp[5] = if x > 2.0 { St::Ok } else if x > 0.9 { St::Run } else { St::Idle };
                sp[3] = if x > 4.9 { St::Back } else if x > 1.2 { St::Run } else { St::Idle };
            }
            Back | Agent2 | ToClaim2 | Claim2 | ToChecks2 => sp = [St::Ok, St::Ok, St::Ok, St::Back, St::Idle, St::Ok],
            Checks2 => {
                let x = f * 5.4;
                sp = [St::Run, St::Run, St::Ok, St::Run, St::Idle, St::Run];
                if x > 0.9 { sp[0] = St::Ok }
                if x > 1.5 { sp[1] = St::Ok }
                if x > 2.0 { sp[5] = St::Ok }
                if x > 4.4 { sp[3] = St::Ok }
                sp[4] = if x > 4.9 { St::You } else if x > 2.4 { St::Run } else { St::Idle };
            }
            ToYou | Waiting => sp = [St::Ok, St::Ok, St::Ok, St::Ok, St::You, St::Ok],
            ToPr | Pr | ToMerged | Merged => sp = [St::Ok; 6],
            _ => {}
        }
        s.spokes = sp;

        // where #123 is
        let lerp = |a: f32, b: f32| a + (b - a) * f;
        s.token = match ph {
            Ticket => (X_TICKET, Y),
            ToAgent => (lerp(X_TICKET, X_AGENT), Y),
            Agent1 | Agent2 => (X_AGENT, Y),
            ToClaim1 | ToClaim2 => (lerp(X_AGENT, X_CLAIM), Y),
            Claim1 | Claim2 => (X_CLAIM, Y),
            ToChecks1 | ToChecks2 => (lerp(X_CLAIM, X_CHECKS), Y),
            Checks1 | Checks2 => (X_CHECKS, Y),
            Back => arc_point(f),
            ToYou => (lerp(X_CHECKS, X_YOU), Y),
            Waiting => (X_YOU, Y),
            ToPr => (lerp(X_YOU, X_PR), Y),
            Pr => (X_PR, Y),
            ToMerged => (lerp(X_PR, X_MERGED), Y),
            Merged => (X_MERGED, Y),
        };
        s.dash = self.t * 22.;
        s.pulse = (self.t * 3.4).sin() * 0.5 + 0.5;
        s
    }

    fn now_line(ph: Phase) -> &'static str {
        use Phase::*;
        match ph {
            Ticket => "Issue #123 comes in: “Fix #123.”",
            ToAgent | Agent1 => "Claude, in the Claude app, writes a failing test, then the fix.",
            ToClaim1 | Claim1 => "It says “done”. That’s a claim, not proof.",
            ToChecks1 | Checks1 => "The checks run on the real stack: unit, types, the merge base and 29 journey runs.",
            Back => "A journey run shows $18.00 on the receipt. Sent back to the agent, not to you.",
            Agent2 => "The agent fixes the receipt. Nothing reached you.",
            ToClaim2 | Claim2 | ToChecks2 => "“Done” again, so it’s checked again.",
            Checks2 => "29 of 29 journey runs are clean. The new layout needs your eye.",
            ToYou | Waiting => "Your turn: one taste call. The agent waits for your answer.",
            ToPr | Pr => "Your answer reaches the agent. The PR says “Closes #123”.",
            ToMerged | Merged => "Merged, with a receipt. You were asked once.",
        }
    }

    fn stage_of(ph: Phase) -> usize {
        use Phase::*;
        match ph {
            Ticket => 0,
            ToAgent | Agent1 | Back | Agent2 => 1,
            ToClaim1 | Claim1 | ToClaim2 | Claim2 => 2,
            ToChecks1 | Checks1 | ToChecks2 | Checks2 => 3,
            ToYou | Waiting => 4,
            ToPr | Pr | ToMerged | Merged => 5,
        }
    }
}

// ---------- drawing ----------
fn line(window: &mut Window, o: Point<Pixels>, a: (f32, f32), b: (f32, f32), w: f32, color: Hsla) {
    let mut pb = PathBuilder::stroke(px(w));
    pb.move_to(point(o.x + px(a.0), o.y + px(a.1)));
    pb.line_to(point(o.x + px(b.0), o.y + px(b.1)));
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}
/// A polyline, solid or dashed; a dashed one marches with `offset` to show work in progress.
fn poly(window: &mut Window, o: Point<Pixels>, pts: &[(f32, f32)], w: f32, color: Hsla, dash: Option<(f32, f32, f32)>) {
    match dash {
        None => {
            for p in pts.windows(2) {
                line(window, o, p[0], p[1], w, color);
            }
        }
        Some((on, off, offset)) => {
            let period = on + off;
            let mut walked = 0.;
            for p in pts.windows(2) {
                let (a, b) = (p[0], p[1]);
                let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
                if len <= 0. { continue; }
                let mut d = 0.;
                while d < len {
                    let phase = (walked + d - offset).rem_euclid(period);
                    let step = if phase < on { (on - phase).min(len - d) } else { (period - phase).min(len - d) };
                    if phase < on {
                        let s = d / len;
                        let e = (d + step) / len;
                        line(window, o, (a.0 + (b.0 - a.0) * s, a.1 + (b.1 - a.1) * s), (a.0 + (b.0 - a.0) * e, a.1 + (b.1 - a.1) * e), w, color);
                    }
                    d += step.max(0.5);
                }
                walked += len;
            }
        }
    }
}
fn paint_map(bounds: Bounds<Pixels>, s: &Snap, pal: &Pal, window: &mut Window) {
    let o = bounds.origin;
    let style = |st: St| -> (f32, Hsla, Option<(f32, f32, f32)>) {
        match st {
            St::Idle => (1.6, pal.border, None),
            St::Run => (2.6, c_run(), Some((8., 6., s.dash))),
            other => (2.6, pal.st(other), None),
        }
    };
    let xs = [X_TICKET, X_AGENT, X_CLAIM, X_CHECKS, X_YOU, X_PR, X_MERGED];
    for i in 0..6 {
        let (w, c, d) = style(s.seg[i]);
        let a = if i == 3 { xs[i] + HUB_R } else { xs[i] + 8. };
        let b = if i == 2 { xs[i + 1] - HUB_R } else { xs[i + 1] - 8. };
        poly(window, o, &[(a, Y), (b, Y)], w, c, d);
    }
    // the sent-back loop
    let pts: Vec<(f32, f32)> = (0..=32).map(|i| arc_point(i as f32 / 32.)).collect();
    let (w, c, d) = if s.arc == St::Back && s.phase == Phase::Back { (2.6, c_back(), Some((8., 6., s.dash))) } else { style(s.arc) };
    poly(window, o, &pts, w, c, d);
    // the checks
    for i in 0..6 {
        let (x, y) = spoke_pos(i);
        let a = (150. - 24. * i as f32).to_radians();
        let from = (X_CHECKS + HUB_R * a.cos(), Y + HUB_R * a.sin());
        let (w, c, d) = style(s.spokes[i]);
        poly(window, o, &[from, (x, y)], w.min(2.2), c, d);
    }
}

impl DoneRight {
    fn node(&self, id: &'static str, x: f32, y: f32, r: f32, st: St, pal: &Pal, pulse: f32, person: bool) -> impl IntoElement {
        let color = pal.st(st);
        let fill = match st { St::Ok => c_ok().opacity(0.14), St::You => c_you().opacity(0.14), St::Back => c_back().opacity(0.14), _ => pal.bg };
        div()
            .absolute()
            .left(px(x - r - 10.))
            .top(px(y - r - 10.))
            .size(px(2. * r + 20.))
            .flex()
            .items_center()
            .justify_center()
            .when(matches!(st, St::Run | St::You), |d| {
                d.child(
                    div()
                        .absolute()
                        .size(px(2. * r + 4. + 12. * pulse))
                        .left(px(r + 10. - (2. * r + 4. + 12. * pulse) / 2.))
                        .top(px(r + 10. - (2. * r + 4. + 12. * pulse) / 2.))
                        .rounded_full()
                        .border_2()
                        .border_color(color.opacity(0.45 * (1. - pulse))),
                )
            })
            .child(
                div()
                    .id(id)
                    .size(px(2. * r))
                    .rounded_full()
                    .border_2()
                    .border_color(color)
                    .bg(fill)
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(person, |d| d.child(Icon::from(Lucide::UserRound).size_3p5().text_color(color)))
                    .when(!person && st == St::Ok, |d| d.child(Icon::from(Lucide::Check).size_3().text_color(c_ok()))),
            )
    }

    fn label(x: f32, y: f32, name: &'static str, sub: &'static str, sub_color: Hsla, pal: &Pal, align_left: bool) -> impl IntoElement {
        div()
            .absolute()
            .left(px(if align_left { x - 12. } else { x - 70. }))
            .top(px(y))
            .w(px(140.))
            .when(!align_left, |d| d.text_center())
            .child(div().text_sm().font_semibold().text_color(pal.fg).child(name))
            .child(div().text_xs().text_color(sub_color).child(sub))
    }

    fn map(&self, s: &Snap, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        let snap = s.clone();
        let p = *pal;
        let st = s.stations;
        let sub_you = match s.phase { Phase::Waiting => "waiting on you", Phase::ToPr | Phase::Pr | Phase::ToMerged | Phase::Merged => "looks right", _ => "taste calls" };
        let you_color = if s.phase == Phase::Waiting { c_you() } else { pal.muted_fg };
        let mut m = div()
            .relative()
            .w(px(W))
            .h(px(H))
            .child(canvas(move |_, _, _| (), move |bounds, _, window, _| paint_map(bounds, &snap, &p, window)).absolute().size_full())
            .child(self.node("st-ticket", X_TICKET, Y, 8., st[0], pal, s.pulse, false))
            .child(self.node("st-agent", X_AGENT, Y, 9., st[1], pal, s.pulse, false))
            .child(self.node("st-claim", X_CLAIM, Y, 8., st[2], pal, s.pulse, false))
            .child(self.node("st-you", X_YOU, Y, 13., st[4], pal, s.pulse, true))
            .child(self.node("st-pr", X_PR, Y, 8., st[5], pal, s.pulse, false))
            .child(self.node("st-merged", X_MERGED, Y, 9., st[6], pal, s.pulse, false))
            // the checks hub
            .child(
                div()
                    .absolute()
                    .left(px(X_CHECKS - HUB_R))
                    .top(px(Y - HUB_R))
                    .size(px(2. * HUB_R))
                    .rounded_full()
                    .border_2()
                    .border_color(pal.st(st[3]))
                    .bg(match st[3] { St::Idle => pal.bg, other => pal.st(other).opacity(0.12) })
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(div().text_xs().font_semibold().text_color(pal.st(st[3])).child("checks")),
            )
            .child(Self::label(X_TICKET, Y + 18., "Issue #123", "“Fix #123.”", pal.muted_fg, pal, true))
            .child(Self::label(X_AGENT, Y + 20., "Claude app", if s.round == 2 { "fixing, round 2" } else { "writes the fix" }, pal.muted_fg, pal, false))
            .child(Self::label(X_CLAIM, Y + 18., "Claim", "“done”", pal.muted_fg, pal, false))
            .child(Self::label(X_YOU, Y + 22., "You", sub_you, you_color, pal, false))
            .child(Self::label(X_PR, Y + 18., "PR", "Closes #123", pal.muted_fg, pal, false))
            .child(Self::label(X_MERGED, Y + 20., "Merged", "with a receipt", pal.muted_fg, pal, false));
        // check names on their spokes
        for (i, name) in CHECKS.iter().enumerate() {
            let (x, y) = spoke_pos(i);
            let c = pal.st(s.spokes[i]);
            let left = i < 2;
            let under = i == 2 || i == 3;
            m = m
                .child(div().absolute().left(px(x - 5.)).top(px(y - 5.)).size(px(10.)).rounded_full().border_2().border_color(c).bg(match s.spokes[i] { St::Idle => pal.bg, o => pal.st(o).opacity(0.2) }))
                .child(
                    div()
                        .absolute()
                        .when(under && i == 2, |d| d.top(px(y + 8.)).left(px(x - 104.)).w(px(110.)).text_right())
                        .when(under && i == 3, |d| d.top(px(y + 8.)).left(px(x - 6.)).w(px(110.)))
                        .when(!under, |d| d.top(px(y - 8.)))
                        .when(left && !under, |d| d.left(px(x - 112.)).w(px(100.)).text_right())
                        .when(!left && !under, |d| d.left(px(x + 12.)).w(px(100.)))
                        .text_xs()
                        .text_color(if s.spokes[i] == St::Idle { pal.muted_fg } else { c })
                        .child(*name),
                );
        }
        // the loop's label, on the arc
        if !s.arc_label.is_empty() {
            let (x, y) = arc_point(0.5);
            m = m.child(
                div()
                    .absolute()
                    .left(px(x - 86.))
                    .top(px(y - 12.))
                    .w(px(172.))
                    .flex()
                    .justify_center()
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_full()
                            .border_1()
                            .border_color(if s.arc_label.starts_with("passed") { c_ok() } else { c_back() })
                            .bg(pal.bg)
                            .text_xs()
                            .text_color(if s.arc_label.starts_with("passed") { c_ok() } else { c_back() })
                            .child(s.arc_label),
                    ),
            );
        }
        // the ticket, moving
        let (tx, ty) = s.token;
        m.child(
            div()
                .absolute()
                .left(px(tx - 24.))
                .top(px(ty - 44.))
                .w(px(48.))
                .flex()
                .justify_center()
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_full()
                        .bg(pal.fg)
                        .text_color(pal.bg)
                        .text_xs()
                        .font_semibold()
                        .shadow_md()
                        .child("#123"),
                ),
        )
        .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, _| {}))
    }

    fn stepper(&self, s: &Snap, cx: &mut Context<Self>) -> impl IntoElement {
        let cur = if s.phase == Phase::Merged { 5 } else { Self::stage_of(s.phase) };
        let steps = [(Lucide::Ticket, "Ticket in"), (Lucide::Bot, "Agent works"), (Lucide::MessageSquareQuote, "Claim"), (Lucide::ListChecks, "Checks"), (Lucide::UserRound, "You"), (Lucide::GitMerge, "Merged")];
        Stepper::new("stages")
            .small()
            .selected_index(cur)
            .items(steps.into_iter().map(|(icon, name)| StepperItem::new().icon(icon).child(name)))
            .on_click(cx.listener(|this, step: &usize, _, cx| {
                this.jump(*step);
                cx.notify();
            }))
    }

    fn legend(pal: &Pal) -> impl IntoElement {
        let item = |c: Hsla, t: &'static str| h_flex().gap_1p5().items_center().child(div().w(px(14.)).h(px(3.)).rounded(px(2.)).bg(c)).child(div().text_xs().text_color(pal.muted_fg).child(t));
        h_flex()
            .gap_3()
            .flex_shrink_0()
            .child(item(c_run(), "in progress"))
            .child(item(c_back(), "sent back"))
            .child(item(c_ok(), "passed"))
            .child(item(c_you(), "needs you"))
            .child(h_flex().gap_1p5().items_center().child(div().w(px(14.)).h(px(8.)).rounded(px(3.)).border_1().border_dashed().border_color(c_added())).child(div().text_xs().text_color(pal.muted_fg).child("added or changed")))
    }

    fn check_rows(&self, s: &Snap, pal: &Pal) -> impl IntoElement {
        v_flex().gap_1().children(CHECKS.iter().enumerate().map(|(i, n)| {
            let st = s.spokes[i];
            let (label, tag) = match st {
                St::Idle => ("not yet", Tag::secondary()),
                St::Run => ("running", Tag::info()),
                St::Ok => ("pass", Tag::success()),
                St::Back => ("fail", Tag::warning()),
                St::You => ("needs you", Tag::danger()),
            };
            let detail = match (i, st) {
                (3, St::Back) => "run 1: receipt shows $18.00",
                (3, St::Ok) => "29 of 29 clean",
                (3, St::Run) => "Playwright, real stack",
                (2, St::Ok) => "the new test fails there, as it must",
                (4, St::You) => "coupon field moved above the total",
                (5, St::Ok) => "6 files, within this repo’s usual size",
                _ => "",
            };
            h_flex()
                .justify_between()
                .gap_2()
                .py_1()
                .border_b_1()
                .border_color(pal.border.opacity(0.6))
                .child(v_flex().child(div().text_sm().child(*n)).when(!detail.is_empty(), |d| d.child(div().text_xs().text_color(pal.muted_fg).child(detail))))
                .child(tag.small().child(label))
        }))
    }

    fn details(&self, s: &Snap, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        let waiting = s.phase == Phase::Waiting;
        let verdict: Option<(&str, Hsla, &str)> = match s.phase {
            Phase::Back | Phase::Agent2 => Some(("FAIL", c_back(), "The agent said the totals match. DoneRight saw $18.00 on the receipt and $18.01 in the cart, in journey run 3 of 29. Sent back to the agent; nothing needs you.")),
            Phase::Waiting => Some(("WAITING ON YOU", c_you(), "5 of 6 checks passed: unit, types, merge base, 29 journey runs and size. Screens needs your eye, and the agent’s stop is held until you answer.")),
            Phase::ToPr | Phase::Pr | Phase::ToMerged | Phase::Merged => Some(("PASS", c_ok(), "Cart and receipt both show $18.01 over 29 runs, and the new test fails on the merge base.")),
            _ => None,
        };
        let receipt = self.idx_of(Kind::Receipt);
        let tab = if receipt.is_some() { self.detail_tab.min(1) } else { 0 };
        v_flex()
            .id("details")
            .w(px(320.))
            .h_full()
            .flex_shrink_0()
            .overflow_y_scroll()
            .border_l_1()
            .border_color(pal.border)
            .bg(pal.card)
            .p_4()
            .gap_2()
            .child(h_flex().gap_2().items_center().child(Tag::primary().small().child("#123")).child(div().text_sm().font_semibold().child("Fix coupon rounding")))
            .child(
                // the list clips its overflow, so in this scrolling column it must not shrink
                div().flex_shrink_0().child(
                    DescriptionList::vertical()
                        .columns(3)
                        .bordered(false)
                        .small()
                        .item("Agent", "Claude app", 1)
                        .item("Repo", "shop", 1)
                        .item("Round", SharedString::from(format!("{} of 3", s.round)), 1),
                ),
            )
            .when_some(verdict, |d, (label, c, body)| {
                d.child(
                    v_flex()
                        .gap_1()
                        .p_3()
                        .rounded(px(10.))
                        .border_1()
                        .border_color(c.opacity(0.5))
                        .bg(c.opacity(0.08))
                        .child(div().text_xs().font_semibold().text_color(c).child(label))
                        .child(div().text_sm().child(body)),
                )
            })
            .when(waiting, |d| {
                d.child(
                    v_flex()
                        .gap_2()
                        .p_3()
                        .rounded(px(10.))
                        .border_1()
                        .border_color(c_you())
                        .child(div().text_xs().font_semibold().text_color(c_you()).child("TASTE CALL"))
                        .child(div().text_sm().font_semibold().child("Does the new checkout layout look right?"))
                        .child(h_flex().gap_2().child(shot("Before", false, pal)).child(shot("After", true, pal)))
                        .child(div().text_xs().child("Look at: the total and the Pay button stay in view after the move."))
                        .child(div().text_xs().text_color(pal.muted_fg).child("Your past calls on checkout: 3 moves like this approved, 1 put back."))
                        .child(
                            h_flex()
                                .gap_2()
                                .child(Button::new("looks").primary().small().label("Looks right").on_click(cx.listener(|this, _, window, cx| this.answer_taste("Looks right", window, cx))))
                                .child(Button::new("back").outline().small().label("Put it back").on_click(cx.listener(|this, _, window, cx| this.answer_taste("Put it back", window, cx)))),
                        ),
                )
            })
            .when_some(self.answer.filter(|_| !waiting), |d, a| d.child(div().text_xs().text_color(pal.muted_fg).child(format!("You answered: {a}"))))
            .child(
                TabBar::new("dtabs")
                    .underline()
                    .small()
                    .selected_index(tab)
                    .on_click(cx.listener(|this, ix: &usize, _, cx| {
                        this.detail_tab = *ix;
                        cx.notify();
                    }))
                    .child(Tab::new().label(format!("Checks · round {}", s.round)))
                    .when(receipt.is_some(), |t| t.child(if self.show_added { Tab::new().label("Receipt").suffix(Icon::from(Lucide::Sparkles).size_3().text_color(c_added())) } else { Tab::new().label("Receipt") })),
            )
            .child(match (tab, receipt) {
                (1, Some(i)) => div().when(self.show_added, |d| d.pt_3()).child(self.mark(i, self.receipt_diff(s, pal), cx)).into_any_element(),
                _ => self.check_rows(s, pal).into_any_element(),
            })
    }

    fn flow_view(&self, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        let s = self.snapshot();
        let speed_btn = |id: &'static str, label: &'static str, v: f32, this: &Self, cx: &mut Context<Self>| {
            Button::new(id).ghost().xsmall().label(label).selected(this.speed == v).on_click(cx.listener(move |this, _, _, cx| { this.speed = v; cx.notify(); }))
        };
        h_flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .px_5()
                            .py_2()
                            .border_b_1()
                            .border_color(pal.border)
                            .child(
                                TabBar::new("flows")
                                    .segmented()
                                    .small()
                                    .selected_index(self.flow_sel)
                                    .on_click(cx.listener(|this, ix: &usize, _, cx| {
                                        this.flow_sel = *ix;
                                        cx.notify();
                                    }))
                                    .children([(Lucide::Ticket, "Fix #123"), (Lucide::Blocks, "Start an app"), (Lucide::Puzzle, "Add anything"), (Lucide::Sparkles, "Suggested")].map(|(icon, label)| {
                                        Tab::new().aria_label(label).child(h_flex().gap_1p5().items_center().child(Icon::from(icon).size_3p5()).child(label))
                                    })),
                            )
                            .when(self.flow_sel == 0, |d| {
                                d.child(
                                    h_flex()
                                        .gap_1()
                                        .child(Button::new("play").ghost().small().icon(if self.playing { Lucide::Pause } else { Lucide::Play }).label(if self.playing { "Pause" } else { "Play" }).on_click(cx.listener(|this, _, _, cx| { this.playing = !this.playing; cx.notify(); })))
                                        .child(Button::new("restart").ghost().small().icon(Lucide::RotateCcw).label("Restart").on_click(cx.listener(|this, _, _, cx| { this.restart(); this.playing = true; cx.notify(); })))
                                        .child(speed_btn("s1", "1×", 1., self, cx))
                                        .child(speed_btn("s2", "2×", 2., self, cx)),
                                )
                            }),
                    )
                    .child(
                        div()
                            .px_5()
                            .pt_4()
                            .child(div().text_lg().font_semibold().child("Now"))
                            .child(div().text_sm().text_color(pal.muted_fg).child(match self.flow_sel {
                                0 => Self::now_line(s.phase),
                                1 => "studio, from one line: Codex’s #4 failed run 3 of its journey and went back to it. Lanes A and C keep going. You’re next at the release.",
                                2 => "screenshot-line improved itself: the 300 ms delay applied on its own. Sending to a chat is new access, so it waits for you.",
                                _ => match self.proposal {
                                    Proposal::Pending => "DoneRight noticed you asking “did it deploy?” 9 times this week. Your agent built a gadget for it in a copy; it waits for one tap.",
                                    Proposal::Accepted => "You added the deploy gadget. It’s on your home, and it improves with use like everything you add.",
                                    Proposal::Dismissed => "You said not now. DoneRight won’t suggest a deploy gadget again this month.",
                                },
                            }))
                            .child({
                                // your version of the shipped Flow view adds this row; it's outlined as yours
                                let also = h_flex()
                                    .gap_2()
                                    .pt_2()
                                    .child(div().text_xs().text_color(pal.muted_fg).child("Also:"))
                                    .child(Tag::secondary().small().child("#131 waits on your sign-in · 40 min"))
                                    .child(Tag::info().small().child("#132 journeys 17 of 29 · 1m 40s"));
                                match self.added.iter().position(|a| !a.removed && a.replaces == Some(View::Flow)) {
                                    Some(i) => div().when(self.show_added, |d| d.pt_3()).child(self.mark(i, also, cx)).into_any_element(),
                                    None => also.into_any_element(),
                                }
                            }),
                    )
                    .child(self.gadget_strip(pal, cx))
                    .child(div().flex_1().flex().items_center().justify_center().min_h_0().child(match self.flow_sel {
                        0 => self.map(&s, pal, cx).into_any_element(),
                        1 => playbook_map(pal, s.dash, s.pulse).into_any_element(),
                        2 => extension_map(pal, s.dash, s.pulse, None).into_any_element(),
                        _ => extension_map(pal, s.dash, s.pulse, Some(self.proposal)).into_any_element(),
                    }))
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .gap_4()
                            .px_5()
                            .py_3()
                            .border_t_1()
                            .border_color(pal.border)
                            .when(self.flow_sel == 0, |d| d.child(div().flex_1().max_w(px(560.)).child(self.stepper(&s, cx))))
                            .when(self.flow_sel != 0, |d| d.child(div().text_xs().text_color(pal.muted_fg).child("Each station shows who does it: an agent, a check, or you.")))
                            .child(Self::legend(pal)),
                    ),
            )
            .child(match self.flow_sel {
                0 => self.details(&s, pal, cx).into_any_element(),
                1 => self.playbook_details(pal).into_any_element(),
                2 => self.extension_details(pal, cx).into_any_element(),
                _ => self.proposal_details(pal, cx).into_any_element(),
            })
    }

    // ---------- first launch ----------
    fn welcome(&self, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        let setting_up = self.screen == Screen::Setup;
        let t = self.setup_t;
        let steps: [(&str, &str, f32, f32); 3] = [
            ("Learn how you work", "212 sessions and 64 notes from Claude Code and Codex, read a slice at a time on this Mac", 0.0, 2.4),
            ("Write your wiki", "38 pages, every line with its source", 2.4, 3.8),
            ("Add DoneRight to your agents", "watch-only hooks in Claude Code and Codex; Codex asks you once to trust them", 3.8, 5.0),
        ];
        div().flex_1().flex().items_center().justify_center().child(
            v_flex()
                .w(px(540.))
                .gap_4()
                .p_8()
                .rounded(px(16.))
                .border_1()
                .border_color(pal.border)
                .bg(pal.card)
                .shadow_lg()
                .child(logo(44., pal))
                .child(div().text_2xl().font_semibold().child("DoneRight"))
                .child(div().text_color(pal.muted_fg).child("Checks your coding agents’ “done” and asks you only what needs a person."))
                .children(steps.iter().map(|(name, sub, a, b)| {
                    let state = if !setting_up { 0 } else if t >= *b { 2 } else if t >= *a { 1 } else { 0 };
                    let pct = if state == 1 { ((t - a) / (b - a) * 100.).clamp(0., 100.) } else if state == 2 { 100. } else { 0. };
                    h_flex()
                        .gap_3()
                        .items_start()
                        .child(
                            div()
                                .mt_0p5()
                                .flex_shrink_0()
                                .size(px(20.))
                                .rounded_full()
                                .border_1()
                                .border_color(match state { 2 => c_ok(), 1 => c_run(), _ => pal.border })
                                .when(state == 2, |d| d.bg(c_ok().opacity(0.15)))
                                .flex()
                                .items_center()
                                .justify_center()
                                .when(state == 2, |d| d.child(Icon::from(Lucide::Check).size_3().text_color(c_ok()))),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_1()
                                .child(div().text_sm().font_semibold().child(*name))
                                .child(div().text_xs().text_color(pal.muted_fg).child(*sub))
                                .when(state == 1, |d| d.child(Progress::new(SharedString::from(format!("p-{name}"))).value(pct))),
                        )
                }))
                .when(!setting_up, |d| {
                    d.child(
                        h_flex()
                            .gap_3()
                            .items_center()
                            .child(Button::new("setup").primary().large().label("Set up").on_click(cx.listener(|this, _, _, cx| { this.screen = Screen::Setup; this.setup_t = 0.; cx.notify(); })))
                            .child(div().text_xs().text_color(pal.muted_fg).child("Nothing leaves your Mac. To remove it, just say so.")),
                    )
                })
                .when(setting_up && t >= 5.0, |d| {
                    d.child(div().text_sm().text_color(c_ok()).child("You’re set. Nothing else to do. Opening your map…"))
                }),
        )
    }
}

fn logo(size: f32, pal: &Pal) -> impl IntoElement {
    div()
        .size(px(size))
        .rounded(px(size * 0.28))
        .bg(pal.primary)
        .flex()
        .items_center()
        .justify_center()
        .child(Icon::from(Lucide::Check).size(px(size * 0.55)).text_color(pal.primary_fg))
}

fn shot(label: &'static str, after: bool, pal: &Pal) -> impl IntoElement {
    v_flex()
        .gap_1()
        .flex_1()
        .child(
            v_flex()
                .gap_1()
                .p_2()
                .h(px(52.))
                .rounded(px(6.))
                .border_1()
                .border_color(pal.border)
                .bg(pal.bg)
                .child(div().h(px(6.)).w(px(60.)).rounded(px(2.)).bg(pal.muted))
                .when(after, |d| d.child(div().h(px(10.)).w_full().rounded(px(2.)).border_1().border_color(c_you()).bg(c_you().opacity(0.1))))
                .child(div().h(px(8.)).w(px(72.)).rounded(px(2.)).bg(pal.muted))
                .when(!after, |d| d.child(div().h(px(10.)).w_full().rounded(px(2.)).bg(pal.muted))),
        )
        .child(div().text_xs().text_color(pal.muted_fg).child(label))
}

fn card(pal: &Pal, c: Hsla, kind: &'static str, title: &'static str, body: &'static str) -> Div {
    v_flex()
        .gap_2()
        .p_4()
        .max_w(px(620.))
        .rounded(px(12.))
        .border_1()
        .border_color(c.opacity(0.6))
        .bg(pal.card)
        .child(div().text_xs().font_semibold().text_color(c).child(kind))
        .child(div().font_semibold().child(title))
        .child(div().text_sm().text_color(pal.muted_fg).child(body))
}

impl Render for DoneRight {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pal = Pal::of(cx);
        let n = self.needs_you();
        let title = TitleBar::new().child(
            h_flex()
                .w_full()
                .pr_3()
                .justify_between()
                .items_center()
                .child(h_flex().gap_2().items_center().child(logo(18., &pal)).child(div().text_sm().font_semibold().child("DoneRight")))
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .when(self.screen == Screen::App, |d| {
                            d.child(div().text_xs().text_color(pal.muted_fg).child("watching Claude Code and Codex · 3 repos"))
                                .child(Tag::danger().small().rounded_full().child(if n == 1 { "1 needs you".to_string() } else { format!("{n} need you") }))
                                .child(div().w(px(1.)).h(px(16.)).bg(pal.border))
                                .child(
                                    Switch::new("show-added")
                                        .small()
                                        .color(c_added())
                                        .checked(self.show_added)
                                        .label("Show what changed")
                                        .tooltip("Outline everything you or your agents added or changed in this app, and where each came from")
                                        .on_click(cx.listener(|this, on: &bool, _, cx| {
                                            this.show_added = *on;
                                            cx.notify();
                                        })),
                                )
                        })
                        .child(
                            Button::new("theme")
                                .ghost()
                                .xsmall()
                                .icon(if self.dark { Lucide::Sun } else { Lucide::Moon })
                                .tooltip(if self.dark { "Light" } else { "Dark" })
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.dark = !this.dark;
                                    Theme::change(if this.dark { ThemeMode::Dark } else { ThemeMode::Light }, Some(window), cx);
                                    cx.notify();
                                })),
                        ),
                ),
        );
        let body = if self.screen != Screen::App {
            self.welcome(&pal, cx).into_any_element()
        } else {
            let nav = |label: &'static str, icon: Lucide, view: View, this: &Self, cx: &mut Context<Self>| {
                SidebarMenuItem::new(label)
                    .icon(icon)
                    .active(this.view == view)
                    .on_click(cx.listener(move |this, _, _, cx| { this.view = view; cx.notify(); }))
            };
            let main = match self.view {
                View::Flow => self.flow_view(&pal, cx).into_any_element(),
                View::NeedsYou => self.needs_full(&pal, cx).into_any_element(),
                View::Sessions => self.sessions_view(&pal, cx).into_any_element(),
                View::Map => self.project_map_view(&pal, cx).into_any_element(),
                View::Add => self.add_view(&pal, cx).into_any_element(),
                View::Learned => self.learned_full(&pal, cx).into_any_element(),
                View::Wiki => self.wiki_view(&pal).into_any_element(),
                View::Numbers => self.numbers_view(&pal).into_any_element(),
                View::Ext(i) if i < self.added.len() && !self.added[i].removed => self.ext_view(i, &pal, cx).into_any_element(),
                View::Ext(_) => self.flow_view(&pal, cx).into_any_element(),
            };
            // what you and your agents added sits in Work with everything else; it's marked only
            // while Show what changed is on, or for a moment when it first appears
            let show = self.show_added;
            let ext_items: Vec<SidebarMenuItem> = self
                .added
                .iter()
                .enumerate()
                .filter(|(_, a)| !a.removed)
                .filter_map(|(i, a)| a.nav.clone().map(|(label, icon)| (i, label, icon, a.age < 6.)))
                .map(|(i, label, icon, fresh)| {
                    SidebarMenuItem::new(label)
                        .icon(icon)
                        .active(self.view == View::Ext(i))
                        .on_click(cx.listener(move |this, _, _, cx| { this.view = View::Ext(i); cx.notify(); }))
                        .suffix(move |_, _| {
                            if show || fresh { added_chip(if fresh && !show { "new" } else { "✦ added" }) } else { div().into_any_element() }
                        })
                })
                .collect();
            let flow_changed = self.added.iter().any(|a| !a.removed && a.replaces == Some(View::Flow));
            let flow_item = nav("Flow", Lucide::Workflow, View::Flow, self, cx);
            let flow_item = if show && flow_changed { flow_item.suffix(|_, _| added_chip("✦ changed")) } else { flow_item };
            let mut work = vec![
                flow_item,
                nav("Needs you", Lucide::Inbox, View::NeedsYou, self, cx).suffix(move |_, _| Tag::danger().small().rounded_full().child(format!("{n}"))),
                nav("Sessions", Lucide::LayoutDashboard, View::Sessions, self, cx),
            ];
            work.extend(ext_items);
            work.push(nav("Map", Lucide::Map, View::Map, self, cx));
            work.push(nav("Numbers", Lucide::Gauge, View::Numbers, self, cx));
            h_flex()
                .flex_1()
                .min_h_0()
                .child(
                    Sidebar::new("nav")
                        .w(px(212.))
                        .header(SidebarHeader::new().child(v_flex().child(div().text_sm().font_semibold().child("acme")).child(div().text_xs().text_color(pal.muted_fg).child("shop · api · docs"))))
                        .child(SidebarGroup::new("Work").child(SidebarMenu::new().children(work)))
                        .child(
                            SidebarGroup::new("Make it yours").child(SidebarMenu::new().children([
                                nav("Add anything", Lucide::Sparkles, View::Add, self, cx),
                                nav("Learned", Lucide::Brain, View::Learned, self, cx),
                                nav("Wiki", Lucide::BookOpen, View::Wiki, self, cx),
                            ])),
                        )
                        .footer(SidebarFooter::new().child(
                            h_flex()
                                .id("tell")
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, window, cx| this.open_palette(None, window, cx)))
                                .w_full()
                                .justify_between()
                                .items_center()
                                .px_2()
                                .py_1p5()
                                .rounded(px(8.))
                                .border_1()
                                .border_color(pal.border)
                                .bg(pal.bg)
                                .text_xs()
                                .text_color(pal.muted_fg)
                                .child("Tell DoneRight…")
                                .when_some(Keystroke::parse("cmd-k").ok(), |d, k| d.child(Kbd::new(k)))
                                .test_support(),
                        )),
                )
                .child(if self.view == View::Flow { main } else { div().id("view").flex_1().h_full().min_w_0().overflow_y_scroll().child(main).into_any_element() })
                .into_any_element()
        };
        v_flex()
            .size_full()
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &OpenPalette, window, cx| this.open_palette(None, window, cx)))
            .bg(pal.bg)
            .text_color(pal.fg)
            .child(title)
            .child(body)
            .when(cfg!(feature = "snap"), |d| {
                // headless snapshots have no window chrome; draw the inactive traffic lights
                d.relative().children([15.75, 38.75, 61.75].map(|x| div().absolute().left(px(x - 6.5)).top(px(9.25)).size(px(13.)).rounded_full().bg(rgb(0xd7d7d7))))
            })
    }
}

fn main() {
    #[cfg(feature = "snap")]
    if std::env::var("DR_SNAP").is_ok() {
        return snap::run();
    }
    gpui_kit::application().with_assets(gpui_kit::assets::AllAssets).run(|cx| {
        gpui_kit::init(cx);
        cx.bind_keys([KeyBinding::new("cmd-k", OpenPalette, None)]);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1440.), px(860.)), cx)),
            ..TitleBar::window_options()
        };
        gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| DoneRight::new(window, cx))).expect("failed to open the window");
    });
}
