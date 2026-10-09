//! The rest of the app: the other flow maps, sessions, the project map, Add anything,
//! the wiki, what it learned, numbers, and every ask that needs you.

use super::*;

// ---------- a small map renderer shared by the playbook and extension flows ----------
pub(crate) struct Stn {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub st: St,
    pub name: &'static str,
    pub sub: &'static str,
    pub person: bool,
    pub hub: bool,
    pub label_above: bool,
}
pub(crate) struct Mini {
    pub w: f32,
    pub h: f32,
    pub stations: Vec<Stn>,
    pub lines: Vec<(Vec<(f32, f32)>, St, bool)>, // points, status, animate when running
    pub spokes: Option<(f32, f32, Vec<(&'static str, St)>, f32, f32)>, // hub x, y, checks, first and last angle
    pub pills: Vec<(f32, f32, &'static str, Hsla)>,
    pub tokens: Vec<(f32, f32, &'static str, Hsla)>,
}

pub(crate) fn cubic(a: (f32, f32), b: (f32, f32), c: (f32, f32), d: (f32, f32), n: usize) -> Vec<(f32, f32)> {
    (0..=n)
        .map(|i| {
            let u = i as f32 / n as f32;
            let v = 1. - u;
            (
                v * v * v * a.0 + 3. * v * v * u * b.0 + 3. * v * u * u * c.0 + u * u * u * d.0,
                v * v * v * a.1 + 3. * v * v * u * b.1 + 3. * v * u * u * c.1 + u * u * u * d.1,
            )
        })
        .collect()
}

fn spoke_at(hx: f32, hy: f32, i: usize, n: usize, r: f32, a0: f32, a1: f32) -> (f32, f32, f32) {
    let a = if n <= 1 { 90f32 } else { a0 + (a1 - a0) * i as f32 / (n - 1) as f32 }.to_radians();
    (hx + r * a.cos(), hy + r * a.sin(), a)
}

pub(crate) fn mini_map(m: Mini, pal: &Pal, dash: f32, pulse: f32) -> Div {
    let p = *pal;
    let lines = m.lines.clone();
    let spokes = m.spokes.clone();
    let mut root = div().relative().w(px(m.w)).h(px(m.h)).child(
        canvas(
            move |_, _, _| (),
            move |bounds, _, window, _| {
                let o = bounds.origin;
                for (pts, st, animate) in &lines {
                    let (w, c, d) = match st {
                        St::Idle => (1.6, p.border, None),
                        St::Run if *animate => (2.6, c_run(), Some((8., 6., dash))),
                        other => (2.6, p.st(*other), None),
                    };
                    poly(window, o, pts, w, c, d);
                }
                if let Some((hx, hy, items, a0, a1)) = &spokes {
                    let n = items.len();
                    for (i, (_, st)) in items.iter().enumerate() {
                        let (x, y, a) = spoke_at(*hx, *hy, i, n, 92., *a0, *a1);
                        let from = (hx + HUB_R * a.cos(), hy + HUB_R * a.sin());
                        let (w, c) = match st { St::Idle => (1.6, p.border), other => (2.2, p.st(*other)) };
                        poly(window, o, &[from, (x, y)], w, c, None);
                    }
                }
            },
        )
        .absolute()
        .size_full(),
    );
    if let Some((hx, hy, items, a0, a1)) = &m.spokes {
        let n = items.len();
        for (i, (name, st)) in items.iter().enumerate() {
            let (x, y, _) = spoke_at(*hx, *hy, i, n, 92., *a0, *a1);
            let c = pal.st(*st);
            let left = x < hx - 8.;
            let mid = (x - hx).abs() <= 8.;
            root = root
                .child(div().absolute().left(px(x - 5.)).top(px(y - 5.)).size(px(10.)).rounded_full().border_2().border_color(c).bg(pal.bg))
                .child(
                    div()
                        .absolute()
                        .top(px(if mid { y + 8. } else { y - 8. }))
                        .when(mid, |d| d.left(px(x - 50.)).w(px(100.)).text_center())
                        .when(left && !mid, |d| d.left(px(x - 112.)).w(px(100.)).text_right())
                        .when(!left && !mid, |d| d.left(px(x + 12.)).w(px(100.)))
                        .text_xs()
                        .text_color(if *st == St::Idle { pal.muted_fg } else { c })
                        .child(*name),
                );
        }
    }
    for s in &m.stations {
        let color = pal.st(s.st);
        let fill = match s.st { St::Ok => c_ok().opacity(0.14), St::You => c_you().opacity(0.14), St::Back => c_back().opacity(0.14), St::Run if s.hub => c_run().opacity(0.12), _ => pal.bg };
        if matches!(s.st, St::Run | St::You) {
            let d = 2. * s.r + 4. + 12. * pulse;
            root = root.child(div().absolute().left(px(s.x - d / 2.)).top(px(s.y - d / 2.)).size(px(d)).rounded_full().border_2().border_color(color.opacity(0.45 * (1. - pulse))));
        }
        root = root.child(
            div()
                .absolute()
                .left(px(s.x - s.r))
                .top(px(s.y - s.r))
                .size(px(2. * s.r))
                .rounded_full()
                .border_2()
                .border_color(color)
                .bg(fill)
                .flex()
                .items_center()
                .justify_center()
                .when(s.hub, |d| d.child(div().text_xs().font_semibold().text_color(color).child(s.name)))
                .when(s.person, |d| d.child(Icon::from(Lucide::UserRound).size_3p5().text_color(color)))
                .when(!s.person && !s.hub && s.st == St::Ok, |d| d.child(Icon::from(Lucide::Check).size_3().text_color(c_ok()))),
        );
        if !s.hub {
            root = root.child(
                div()
                    .absolute()
                    .left(px(s.x - 70.))
                    .top(px(if s.label_above { s.y - s.r - 38. } else { s.y + s.r + 6. }))
                    .w(px(140.))
                    .text_center()
                    .child(div().text_sm().font_semibold().text_color(pal.fg).child(s.name))
                    .child(div().text_xs().text_color(if s.st == St::You { c_you() } else if s.st == St::Back { c_back() } else { pal.muted_fg }).child(s.sub)),
            );
        }
    }
    for (x, y, text, c) in &m.pills {
        root = root.child(
            div().absolute().left(px(x - 110.)).top(px(y - 11.)).w(px(220.)).flex().justify_center().child(
                div().px_2().py_0p5().rounded_full().border_1().border_color(*c).bg(pal.bg).text_xs().text_color(*c).child(*text),
            ),
        );
    }
    for (x, y, text, c) in &m.tokens {
        root = root.child(
            div().absolute().left(px(x - 30.)).top(px(y - 42.)).w(px(60.)).flex().justify_center().child(
                div().px_2().py_0p5().rounded_full().bg(*c).text_color(pal.bg).text_xs().font_semibold().shadow_md().child(*text),
            ),
        );
    }
    root
}

// ---------- the playbook flow: one line in, lanes in parallel, proof, you, release ----------
pub(crate) fn playbook_map(pal: &Pal, dash: f32, pulse: f32) -> Div {
    let y = 132.;
    let (fan, join) = (186., 420.);
    let lane = |ly: f32| {
        let mut pts = cubic((fan, y), (fan + 24., y), (fan + 12., ly), (fan + 40., ly), 12);
        pts.extend([(join - 40., ly)]);
        pts.extend(cubic((join - 40., ly), (join - 12., ly), (join - 24., y), (join, y), 12));
        pts
    };
    let lanes = [(62., St::Ok, "Lane A · Claude", "3 of 3 merged"), (132., St::Back, "Lane B · Codex", "#4 sent back once"), (202., St::Run, "Lane C · Claude", "#6 building")];
    let mut lines = vec![
        (vec![(52., y), (142., y)], St::Ok, false),
        (vec![(142., y), (fan, y)], St::Ok, false),
        (vec![(join, y), (468., y)], St::Run, true),
        (vec![(516., y), (630., y)], St::Idle, false),
        (vec![(630., y), (740., y)], St::Idle, false),
        (vec![(740., y), (830., y)], St::Idle, false),
    ];
    for (ly, st, _, _) in lanes {
        lines.push((lane(ly), if st == St::Ok { St::Ok } else { St::Run }, st != St::Ok));
    }
    let mut stations = vec![
        Stn { x: 52., y, r: 8., st: St::Ok, name: "Research", sub: "14 sources", person: false, hub: false, label_above: false },
        Stn { x: 142., y, r: 12., st: St::Ok, name: "Plan", sub: "you approved", person: true, hub: false, label_above: false },
        Stn { x: 492., y, r: HUB_R, st: St::Run, name: "prove", sub: "", person: false, hub: true, label_above: false },
        Stn { x: 630., y, r: 8., st: St::Idle, name: "Merge", sub: "in a safe order", person: false, hub: false, label_above: true },
        Stn { x: 740., y, r: 12., st: St::Idle, name: "Release", sub: "you approve", person: true, hub: false, label_above: true },
        Stn { x: 830., y, r: 8., st: St::Idle, name: "Live", sub: "checked live", person: false, hub: false, label_above: true },
    ];
    for (ly, st, name, sub) in lanes {
        stations.push(Stn { x: 302., y: ly, r: 8., st, name, sub, person: false, hub: false, label_above: true });
    }
    mini_map(
        Mini {
            w: 880.,
            h: 300.,
            stations,
            lines,
            spokes: Some((492., y, vec![("journeys ×29", St::Run), ("screens", St::Idle), ("merge base", St::Ok), ("size", St::Ok)], 100., 10.)),
            pills: vec![],
            tokens: vec![(362., 132. + 30., "#4", c_back()), (362., 202. + 30., "#6", c_run())],
        },
        pal,
        dash,
        pulse,
    )
}

// ---------- an extension build: words in, your agent, checks, you, your Mac, and back ----------
// `suggested` draws the same line for something DoneRight proposed from how you work:
// the words come from a pattern it noticed, and the build happens in a copy.
pub(crate) fn extension_map(pal: &Pal, dash: f32, pulse: f32, suggested: Option<Proposal>) -> Div {
    let y = 120.;
    let (ask, agent, checks, you, mac) = (56., 196., 372., 548., 700.);
    let arc = cubic((checks, y - HUB_R), (checks, y - 100.), (agent, y - 100.), (agent, y - 9.), 32);
    let back_loop = {
        let mut v = vec![(mac, y + 10.), (mac, y + 150.)];
        v.extend([(130., y + 150.), (130., y)]);
        v
    };
    let Some(p) = suggested else {
        return mini_map(
            Mini {
                w: 860.,
                h: 300.,
                stations: vec![
                    Stn { x: ask, y, r: 8., st: St::Ok, name: "You asked", sub: "in the app", person: false, hub: false, label_above: false },
                    Stn { x: agent, y, r: 9., st: St::Ok, name: "Your agent", sub: "on your plan", person: false, hub: false, label_above: false },
                    Stn { x: checks, y, r: HUB_R, st: St::Ok, name: "checks", sub: "", person: false, hub: true, label_above: false },
                    Stn { x: you, y, r: 13., st: St::You, name: "You", sub: "1 change waits", person: true, hub: false, label_above: false },
                    Stn { x: mac, y, r: 9., st: St::Ok, name: "On your Mac", sub: "screenshot-line 0.1.1", person: false, hub: false, label_above: true },
                ],
                lines: vec![
                    (vec![(ask, y), (agent, y)], St::Ok, false),
                    (vec![(agent, y), (checks - HUB_R, y)], St::Ok, false),
                    (vec![(checks + HUB_R, y), (you, y)], St::Ok, false),
                    (vec![(you, y), (mac, y)], St::Ok, false),
                    (arc, St::Back, false),
                    (back_loop, St::Run, true),
                ],
                spokes: Some((checks, y, vec![("conformance", St::Ok), ("its tests", St::Ok), ("render", St::Ok), ("replay ×61", St::Ok), ("access", St::You)], 135., 45.)),
                pills: vec![((checks + agent) / 2., y - 76., "passed on round 2", c_ok()), (400., y + 150., "improves with use · replayed on your past uses", c_run())],
                tokens: vec![(you, y, "0.2", c_you())],
            },
            pal,
            dash,
            pulse,
        );
    };
    let (you_st, you_sub, home_st, home_sub) = match p {
        Proposal::Pending => (St::You, "one tap", St::Idle, "a gadget, when you say"),
        Proposal::Accepted => (St::Ok, "you added it", St::Ok, "Deploys gadget"),
        Proposal::Dismissed => (St::Ok, "you said not now", St::Idle, "not added"),
    };
    mini_map(
        Mini {
            w: 860.,
            h: 300.,
            stations: vec![
                Stn { x: ask, y, r: 8., st: St::Ok, name: "Noticed", sub: "“did it deploy?” ×9", person: false, hub: false, label_above: false },
                Stn { x: agent, y, r: 9., st: St::Ok, name: "Your agent", sub: "builds it in a copy", person: false, hub: false, label_above: false },
                Stn { x: checks, y, r: HUB_R, st: St::Ok, name: "checks", sub: "", person: false, hub: true, label_above: false },
                Stn { x: you, y, r: 13., st: you_st, name: "You", sub: you_sub, person: true, hub: false, label_above: false },
                Stn { x: mac, y, r: 9., st: home_st, name: "Your home", sub: home_sub, person: false, hub: false, label_above: true },
            ],
            lines: vec![
                (vec![(ask, y), (agent, y)], St::Ok, false),
                (vec![(agent, y), (checks - HUB_R, y)], St::Ok, false),
                (vec![(checks + HUB_R, y), (you, y)], St::Ok, false),
                (vec![(you, y), (mac, y)], if p == Proposal::Accepted { St::Ok } else { St::Idle }, false),
                (back_loop, if p == Proposal::Accepted { St::Run } else { St::Idle }, true),
            ],
            spokes: Some((checks, y, vec![("conformance", St::Ok), ("its tests", St::Ok), ("render", St::Ok), ("dry run", St::Ok), ("access", St::Ok)], 135., 30.)),
            pills: vec![((checks + agent) / 2., y - 76., "passed first time", c_ok()), (400., y + 150., "improves with use, like anything you add", if p == Proposal::Accepted { c_run() } else { pal.muted_fg })],
            tokens: if p == Proposal::Pending { vec![(you, y, "new", c_you())] } else { vec![] },
        },
        pal,
        dash,
        pulse,
    )
}

// ---------- shared pieces ----------
pub(crate) fn section(title: &'static str, pal: &Pal) -> Div {
    div().pt_2().text_xs().font_semibold().text_color(pal.muted_fg).child(title)
}
pub(crate) fn box_(pal: &Pal) -> Div {
    v_flex().gap_2().p_4().rounded(px(12.)).border_1().border_color(pal.border).bg(pal.card)
}
fn scope_ladder(scope: usize, repo: &'static str, pal: &Pal) -> impl IntoElement {
    let item = |i: usize, t: &'static str| {
        div()
            .px_2()
            .py_0p5()
            .text_xs()
            .when(i == scope, |d| d.bg(c_run().opacity(0.12)).text_color(c_run()).font_semibold())
            .when(i != scope, |d| d.text_color(pal.muted_fg))
            .when(i > 0, |d| d.border_l_1().border_color(pal.border))
            .child(t)
    };
    h_flex().rounded(px(999.)).border_1().border_color(pal.border).overflow_hidden().child(item(0, "session")).child(item(1, repo)).child(item(2, "everywhere"))
}

// ---------- sessions: every agent, every repo, with controls ----------
pub(crate) struct Ses {
    pub id: &'static str,
    pub repo: &'static str,
    pub agent: &'static str,
    pub what: &'static str,
    pub doing: &'static str,
    pub st: St,
    pub flag: Option<&'static str>,
    /// how long it has been in this state
    pub since: &'static str,
    /// a status word other than the state's own, such as "checking" after the agent says done
    pub label: Option<&'static str>,
}
pub(crate) fn sessions() -> Vec<Ses> {
    vec![
        Ses { id: "coupon", repo: "shop", agent: "Claude app", what: "#123 Fix coupon rounding", doing: "waiting on your taste call", st: St::You, flag: None, since: "12 min", label: None },
        Ses { id: "search", repo: "shop", agent: "Claude app", what: "#132 Search keeps the price filter", doing: "journeys 17 of 29 · 1m 40s", st: St::Run, flag: None, since: "4 min", label: None },
        Ses { id: "search2", repo: "shop", agent: "Codex app", what: "#132 Search keeps the price filter", doing: "editing src/search/filters.ts", st: St::Run, flag: Some("same issue as Claude · search"), since: "9 min", label: None },
        Ses { id: "tax", repo: "shop", agent: "Codex app", what: "#131 Show tax on the receipt", doing: "blocked on shopper-2’s sign-in", st: St::Back, flag: None, since: "40 min", label: None },
        Ses { id: "export", repo: "shop", agent: "Claude app", what: "#133 Export orders to CSV", doing: "PASS · 6 of 6 checks · PR open", st: St::Ok, flag: None, since: "1 h", label: None },
        Ses { id: "rate", repo: "api", agent: "Codex app", what: "#88 Rate-limit the export", doing: "says done · unit and types passed, journeys 3 of 12", st: St::Run, flag: None, since: "2 min", label: Some("checking") },
        Ses { id: "auth", repo: "api", agent: "Claude app", what: "#91 Rotate service tokens", doing: "stopped by a usage limit; resumes after 2:55 pm", st: St::Back, flag: Some("interrupted"), since: "25 min", label: None },
        Ses { id: "guide", repo: "docs", agent: "Claude app", what: "Update the setup guide", doing: "quiet since 1:10 pm · nothing pushed", st: St::Idle, flag: Some("not pushed"), since: "2 h", label: None },
    ]
}

impl DoneRight {
    pub(crate) fn sessions_view(&self, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        let all = sessions();
        let mut col = v_flex()
            .flex_1()
            .h_full()
            .p_6()
            .gap_3()
            .child(h_flex().justify_between().items_center().child(div().text_lg().font_semibold().child("Every session, by repo")).child(div().text_xs().text_color(pal.muted_fg).child("8 sessions in 3 repos · what needs you and what’s stuck first")));
        for repo in ["shop", "api", "docs"] {
            let rows: Vec<&Ses> = all.iter().filter(|s| s.repo == repo).collect();
            col = col.child(section(match repo { "shop" => "SHOP · 5", "api" => "API · 2", _ => "DOCS · 1" }, pal));
            for s in rows {
                let paused = self.paused.contains(s.id);
                let noted = self.noted.contains(s.id);
                let id = s.id;
                let (label, tag) = if paused {
                    ("paused by you", Tag::secondary())
                } else {
                    match (s.st, s.label) {
                        (St::Run, Some(l)) => (l, Tag::info()),
                        (St::You, _) => ("needs you", Tag::danger()),
                        (St::Run, _) => ("working", Tag::info()),
                        (St::Back, _) => ("stuck", Tag::warning()),
                        (St::Ok, _) => ("done", Tag::success()),
                        (St::Idle, _) => ("idle", Tag::secondary()),
                    }
                };
                col = col.child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .py_2()
                        .px_3()
                        .rounded(px(10.))
                        .border_1()
                        .border_color(if s.flag.is_some() && !paused { c_back().opacity(0.5) } else { pal.border })
                        .child(Icon::from(Lucide::Bot).size_4().text_color(pal.muted_fg))
                        .child(div().w(px(84.)).text_sm().child(s.agent))
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .child(div().text_sm().font_semibold().child(s.what))
                                .child(div().text_xs().text_color(pal.muted_fg).child(if paused { "its next tool call waits until you resume it" } else if noted { "your note goes in at its next stop" } else { s.doing })),
                        )
                        .when(s.st == St::Idle && !paused, |d| d.opacity(0.72))
                        .when_some(s.flag.filter(|_| !paused), |d, f| d.child(Tag::warning().small().outline().child(f)))
                        .when(!paused, |d| d.child(div().w(px(44.)).text_xs().text_color(pal.muted_fg).text_right().child(s.since)))
                        .child(tag.small().child(label))
                        .child(Button::new(SharedString::from(format!("open-{id}"))).ghost().xsmall().icon(Lucide::ExternalLink).label("Open"))
                        .child(
                            Button::new(SharedString::from(format!("pause-{id}")))
                                .outline()
                                .xsmall()
                                .label(if paused { "Resume" } else { "Pause" })
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    if this.paused.remove(id) {
                                        toast(window, cx, "Resumed", "It carries on from where it stopped.");
                                    } else {
                                        this.paused.insert(id);
                                        toast(window, cx, "Paused", "Its next tool call waits until you resume it.");
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new(SharedString::from(format!("note-{id}")))
                                .ghost()
                                .xsmall()
                                .label(if noted { "Noted" } else { "Note" })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.noted.insert(id);
                                    cx.notify();
                                })),
                        ),
                );
            }
        }
        col.child(div().text_xs().text_color(pal.muted_fg).child("Open jumps to the session in its own app. Pause holds its next tool call. A note reaches it at its next stop."))
    }

    // ---------- the project map: each part with who's there and what's proven ----------
    pub(crate) fn project_map_view(&self, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        struct Part { name: &'static str, path: &'static str, sessions: &'static [&'static str], issues: &'static [&'static str], ev: &'static [(&'static str, St)] }
        let shop = [
            Part { name: "checkout", path: "src/checkout · src/cart", sessions: &["Claude · coupon"], issues: &["#123"], ev: &[("journey checkout · 29 of 29", St::Ok), ("unit · 214 tests", St::Ok)] },
            Part { name: "search", path: "src/search", sessions: &["Claude · search", "Codex · search-2"], issues: &["#132"], ev: &[("journey search · 10 of 12, a known failure", St::Run)] },
            Part { name: "receipts", path: "src/receipts", sessions: &["Codex · tax"], issues: &["#131"], ev: &[("unit · 38 tests", St::Ok), ("phone check · waits on a sign-in", St::You)] },
            Part { name: "export", path: "src/export", sessions: &["Claude · export"], issues: &["#133"], ev: &[("unit · 12 tests", St::Ok), ("CSV journey · 3 of 3", St::Ok)] },
            Part { name: "account", path: "src/account", sessions: &[], issues: &[], ev: &[("unit · 38 tests", St::Ok), ("no journey", St::Back)] },
            Part { name: "content", path: "src/content", sessions: &[], issues: &[], ev: &[("no test touches it", St::Back)] },
        ];
        let api = [
            Part { name: "export API", path: "api/export", sessions: &["Codex · rate"], issues: &["#88"], ev: &[("unit · 44 tests", St::Ok), ("load check · not yet", St::Idle)] },
            Part { name: "auth", path: "api/auth", sessions: &["Claude · auth"], issues: &["#91"], ev: &[("unit · 61 tests", St::Ok)] },
            Part { name: "db", path: "db/migrations", sessions: &[], issues: &[], ev: &[("migrations apply", St::Ok)] },
        ];
        let docs = [Part { name: "guides", path: "docs/", sessions: &["Claude · guide"], issues: &[], ev: &[("links resolve", St::Ok)] }];
        let parts: &[Part] = match self.map_repo { 0 => &shop, 1 => &api, _ => &docs };
        v_flex()
            .flex_1()
            .h_full()
            .p_6()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(div().text_lg().font_semibold().child("Map: which part is moving, and what’s proven"))
                    .child(
                        TabBar::new("repos")
                            .segmented()
                            .small()
                            .selected_index(self.map_repo)
                            .on_click(cx.listener(|this, ix: &usize, _, cx| {
                                this.map_repo = *ix;
                                cx.notify();
                            }))
                            .child(Tab::new().label("shop"))
                            .child(Tab::new().label("api"))
                            .child(Tab::new().label("docs")),
                    ),
            )
            .child(
                h_flex().flex_wrap().items_start().gap_3().children(parts.iter().map(|p| {
                    let gap = p.ev.iter().any(|(_, s)| *s == St::Back);
                    v_flex()
                        .w(px(276.))
                        .gap_1p5()
                        .p_3()
                        .rounded(px(12.))
                        .border_1()
                        .border_color(if gap { c_back().opacity(0.6) } else { pal.border })
                        .bg(pal.card)
                        .child(h_flex().justify_between().child(div().font_semibold().child(p.name)).when(gap, |d| d.child(Tag::warning().small().child("gap"))))
                        .child(div().text_xs().text_color(pal.muted_fg).child(p.path))
                        .child(h_flex().flex_wrap().gap_1().children(p.sessions.iter().map(|s| Tag::info().small().child(*s))).children(p.issues.iter().map(|s| Tag::secondary().small().child(*s))))
                        .children(p.ev.iter().map(|(t, s)| {
                            h_flex().gap_1p5().items_center().child(div().size(px(7.)).rounded_full().bg(pal.st(*s))).child(div().text_xs().child(*t))
                        }))
                })),
            )
            .child(div().text_xs().text_color(pal.muted_fg).child("Parts come from the repo’s own structure. With Understand-Anything’s graph in a repo, the map reads it too."))
    }

    // ---------- Add anything: your words, built by your own agent ----------
    pub(crate) fn add_view(&self, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        let builds = self.builds.clone();
        v_flex()
            .flex_1()
            .h_full()
            .p_6()
            .gap_3()
            .child(div().text_lg().font_semibold().child("Add anything"))
            .child(
                box_(pal)
                    .child(div().text_sm().text_color(pal.muted_fg).child("Say what you want. Your own agent builds it on your plan, DoneRight checks it, and you approve once."))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().flex_1().child(Input::new(&self.input)))
                            .child(Button::new("build").primary().label("Build it").on_click(cx.listener(|this, _, window, cx| this.submit(window, cx)))),
                    )
                    .child(h_flex().gap_2().children(["Hang every screenshot on a line under the menu bar", "Watch the receipts inbox for bounces", "Point at things on my screen when I hold ⌥Space"].into_iter().enumerate().map(|(i, t)| {
                        Button::new(("idea", i)).ghost().xsmall().label(t).on_click(cx.listener(move |this, _, _, cx| { this.start_build(t.to_string()); cx.notify(); }))
                    }))),
            )
            .children(builds.into_iter().enumerate().map(|(i, b)| {
                let stage = if b.installed { 3 } else if b.t < 3.0 { 0 } else if b.t < 6.0 { 1 } else { 2 };
                box_(pal)
                    .child(h_flex().justify_between().child(div().font_semibold().child(format!("“{}”", b.words))).child(match stage {
                        3 => Tag::success().small().child("added"),
                        2 => Tag::danger().small().child("ready for you"),
                        _ => Tag::info().small().child("on your plan"),
                    }))
                    .child(
                        Stepper::new(("bs", i))
                            .small()
                            .selected_index(stage)
                            .items(["Your agent builds it", "Checks", "You", "In the app"].into_iter().map(|t| StepperItem::new().child(t))),
                    )
                    .child(match stage {
                        0 => ShimmerText::new("Claude Code, headless, on your Max plan: writing the panel script, its grant and its tests").id(("sh", i)).text_xs().into_any_element(),
                        1 => ShimmerText::new("conformance · its own tests · a render · a dry run on your data · an access check").id(("sh", i)).text_xs().into_any_element(),
                        2 => div().text_xs().text_color(pal.muted_fg).child("All checks passed. It can read only what it declared, and it can’t send anything.").into_any_element(),
                        _ => div().text_xs().text_color(pal.muted_fg).child("In Work in your sidebar, next to Sessions, loaded into the app with no rebuild.").into_any_element(),
                    })
                    .when(stage == 2, |d| {
                        d.child(
                            h_flex()
                                .gap_2()
                                .child(Button::new(("install", i)).primary().small().label("Add it to the app").on_click(cx.listener(move |this, _, window, cx| this.install_build(i, window, cx))))
                                .child(Button::new(("skip", i)).outline().small().label("Not now")),
                        )
                    })
                    .when_some(b.added.filter(|_| stage == 3), |d, idx| {
                        d.child(h_flex().child(Button::new(("show", i)).outline().small().icon(Lucide::ArrowRight).label("Show me").on_click(cx.listener(move |this, _, _, cx| { this.view = View::Ext(idx); cx.notify(); }))))
                    })
            }))
            .when(self.proposal == Proposal::Pending, |d| {
                d.child(section("SUGGESTED FROM HOW YOU WORK", pal)).child(self.proposal_card(pal, cx))
            })
            .child(section("IN THE APP NOW · BUILT FROM ONE SENTENCE EACH", pal))
            .child(
                h_flex().gap_3().flex_wrap().items_start().children(self.added.iter().enumerate().filter(|(_, a)| !a.removed).map(|(i, a)| {
                    v_flex()
                        .w(px(300.))
                        .gap_1()
                        .p_3()
                        .rounded(px(12.))
                        .border_1()
                        .border_color(pal.border)
                        .child(h_flex().gap_1p5().items_center().child(div().text_sm().font_semibold().child(format!("{} {}", a.key, a.version))).when(a.suggested, |d| d.child(Tag::secondary().small().child("suggested"))))
                        .child(div().text_xs().text_color(pal.muted_fg).child(format!("“{}”", a.words)))
                        .child(div().text_xs().child(format!("In: {}", a.places())))
                        .child(
                            h_flex()
                                .gap_1()
                                .pt_1()
                                .child(Button::new(("about", i)).ghost().xsmall().icon(Lucide::Info).label("Where it came from").on_click(cx.listener(move |this, _, window, cx| this.open_about(i, window, cx))))
                                .when(a.nav.is_some(), |d| d.child(Button::new(("go", i)).ghost().xsmall().icon(Lucide::ArrowRight).label("Open").on_click(cx.listener(move |this, _, _, cx| { this.view = View::Ext(i); cx.notify(); })))),
                        )
                })),
            )
            .child(section("TRY SIDE BY SIDE", pal))
            .child(self.memory_trial(pal, cx))
    }

    /// The suggestion as a card, where it waits until you look: Needs you and Add anything.
    pub(crate) fn proposal_card(&self, pal: &Pal, cx: &mut Context<Self>) -> Div {
        box_(pal)
            .max_w(px(620.))
            .border_dashed()
            .border_color(c_added())
            .child(div().text_xs().font_semibold().text_color(c_added()).child("SUGGESTED GADGET · NO RUSH"))
            .child(div().font_semibold().child("Add a deploy gadget to your home?"))
            .child(div().text_sm().text_color(pal.muted_fg).child("You asked “did it deploy?” 9 times this week, in 4 sessions. Your agent built this in a copy on your plan (1% of one window), and its checks passed. It reads deploy status with your Vercel and Fly logins, read-only."))
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(v_flex().w(px(206.)).p_3().gap_1p5().rounded(px(12.)).border_1().border_color(pal.border).bg(pal.bg).child(h_flex().gap_1p5().items_center().child(Icon::from(Lucide::Rocket).size_3p5().text_color(pal.muted_fg)).child(div().text_xs().font_semibold().text_color(pal.muted_fg).child("Deploys · preview"))).child(deploy_preview(pal)))
                    .child(
                        v_flex()
                            .gap_2()
                            .child(Button::new("pc-add").primary().small().label("Add it").on_click(cx.listener(|this, _, window, cx| this.accept_proposal(window, cx))))
                            .child(Button::new("pc-no").outline().small().label("Not now").on_click(cx.listener(|this, _, window, cx| this.dismiss_proposal(window, cx)))),
                    ),
            )
    }

    pub(crate) fn proposal_details(&self, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w(px(320.))
            .h_full()
            .flex_shrink_0()
            .border_l_1()
            .border_color(pal.border)
            .bg(pal.card)
            .p_4()
            .gap_2()
            .child(h_flex().gap_2().items_center().child(Tag::secondary().small().child("suggested")).child(div().text_sm().font_semibold().child("deploy-status 0.1.0")))
            .child(div().text_xs().text_color(pal.muted_fg).child("Nobody asked for this one. DoneRight noticed a pattern and asked your agent to build it, in a copy, on your plan."))
            .child(section("WHY", pal))
            .child(div().text_xs().child("You asked “did it deploy?” 9 times this week, in 4 sessions, and opened the Vercel dashboard 6 times."))
            .child(section("PREVIEW, LIVE FROM THE COPY", pal))
            .child(v_flex().p_3().rounded(px(12.)).border_1().border_dashed().border_color(c_added()).bg(pal.bg).child(deploy_preview(pal)))
            .child(section("IT CAN", pal))
            .child(div().text_xs().child("✓ read deploy status with your Vercel and Fly logins, read-only"))
            .child(section("IT CAN’T", pal))
            .children(["deploy, roll back or change anything", "use any other login"].into_iter().map(|t| div().text_xs().text_color(pal.muted_fg).child(format!("✗ {t}"))))
            .child(div().h(px(4.)))
            .child(match self.proposal {
                Proposal::Pending => h_flex()
                    .gap_2()
                    .child(Button::new("pd-add").primary().small().label("Add it").on_click(cx.listener(|this, _, window, cx| this.accept_proposal(window, cx))))
                    .child(Button::new("pd-no").outline().small().label("Not now").on_click(cx.listener(|this, _, window, cx| this.dismiss_proposal(window, cx))))
                    .into_any_element(),
                Proposal::Accepted => div().text_xs().text_color(c_ok()).child("Added. It’s on your home screen.").into_any_element(),
                Proposal::Dismissed => div().text_xs().text_color(pal.muted_fg).child("You said not now.").into_any_element(),
            })
            .child(div().text_xs().text_color(pal.muted_fg).child("Suggestions never count as asks. They wait here and in Needs you until you look."))
    }

    pub(crate) fn memory_trial(&self, pal: &Pal, cx: &mut Context<Self>) -> Div {
        let rows = [
            ("Facts you had to repeat that it would have given", "7 of 12", "10 of 12"),
            ("Wrong or stale facts", "none", "2"),
            ("Added to every session start", "410 tokens", "1,240 tokens"),
            ("Cost this week", "$0", "$0.38"),
        ];
        box_(pal)
            .child(h_flex().justify_between().child(div().font_semibold().child("Your wiki and Hindsight, a week as copies")).child(match self.memory {
                Some(c) => Tag::success().small().child(c),
                None => Tag::danger().small().child("your call"),
            }))
            .child(
                v_flex().children(rows.into_iter().map(|(k, a, b)| {
                    h_flex().py_1().border_b_1().border_color(pal.border.opacity(0.6)).child(div().flex_1().text_xs().text_color(pal.muted_fg).child(k)).child(div().w(px(110.)).text_sm().child(a)).child(div().w(px(110.)).text_sm().child(b))
                })),
            )
            .when(self.memory.is_none(), |d| {
                d.child(
                    h_flex()
                        .gap_2()
                        .child(Button::new("m-both").primary().small().label("Both: wiki first").on_click(cx.listener(|this, _, _, cx| { this.memory = Some("both, wiki first"); cx.notify(); })))
                        .child(Button::new("m-switch").outline().small().label("Switch to Hindsight").on_click(cx.listener(|this, _, _, cx| { this.memory = Some("switched to Hindsight"); cx.notify(); })))
                        .child(Button::new("m-wiki").ghost().small().label("Keep my wiki").on_click(cx.listener(|this, _, _, cx| { this.memory = Some("kept your wiki"); cx.notify(); }))),
                )
            })
    }

    // ---------- the wiki: what it knows, with sources ----------
    pub(crate) fn wiki_view(&self, pal: &Pal) -> impl IntoElement {
        let groups = [
            ("you", 9, "what you check before “done”, your taste calls, your standing answers"),
            ("repos", 14, "shop, api and docs: how each starts, its tests, CI and journeys"),
            ("setup", 8, "ports, test accounts, credentials by reference (names only)"),
            ("rules", 7, "no kills by port, no bare git stash, tests without the production .env"),
        ];
        let claims = [
            ("A fix starts with a failing test.", "you asked for it in 14 sessions · CLAUDE.md line 12"),
            ("shop starts with npm run dev on a leased port; ready at /health.", "package.json · 31 sessions"),
            ("STRIPE_TEST_KEY comes from ~/.config/shop/stripe.env, by reference.", "30 of 32 sessions"),
        ];
        v_flex()
            .flex_1()
            .h_full()
            .p_6()
            .gap_3()
            .child(h_flex().justify_between().child(div().text_lg().font_semibold().child("Your wiki")).child(div().text_xs().text_color(pal.muted_fg).child("38 pages of markdown in ~/.doneright/wiki · edit them anywhere")))
            .child(h_flex().gap_3().flex_wrap().items_start().children(groups.into_iter().map(|(g, n, what)| {
                v_flex().w(px(240.)).gap_1().p_3().rounded(px(12.)).border_1().border_color(pal.border).child(h_flex().justify_between().child(div().font_semibold().child(g)).child(Tag::secondary().small().child(format!("{n} pages")))).child(div().text_xs().text_color(pal.muted_fg).child(what))
            })))
            .child(section("EVERY LINE NAMES ITS SOURCE", pal))
            .children(claims.into_iter().map(|(c, src)| {
                h_flex().gap_3().py_1p5().border_b_1().border_color(pal.border.opacity(0.6)).child(div().flex_1().text_sm().child(c)).child(div().text_xs().text_color(pal.muted_fg).child(src))
            }))
            .child(section("WAITING UNTIL THEY MATTER · 3", pal))
            .children([
                ("Which way api starts", "asked the first time an agent starts api"),
                ("Screenshots on hotfix branches too?", "asked on your first hotfix that changes a screen"),
                ("Where STRIPE_TEST_KEY comes from", "asked with shop’s checks"),
            ].into_iter().map(|(q, w)| h_flex().gap_3().py_1().child(div().flex_1().text_sm().child(q)).child(div().text_xs().text_color(pal.muted_fg).child(w))))
            .child(div().text_xs().text_color(pal.muted_fg).child("Your agent read your history a slice at a time: 212 sessions in 38 small sub-reads, about 4% of one 5-hour window."))
    }

    // ---------- what it learned, local first, with a way back ----------
    pub(crate) fn learned_full(&self, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        let items: [(&'static str, &'static str, &'static str, usize, &'static str, &'static str); 4] = [
            ("wiki", "WIKI", "shop’s journeys use Stripe test mode, never a mock", 1, "shop", "3 sessions · 4 quotes"),
            ("ids", "SESSION START", "Agents in shop get the checkout page’s test ids", 1, "shop", "2 sessions where agents hunted for them"),
            ("delay", "EXTENSION", "screenshot-line 0.1.1: wait 300 ms before the line glides down", 2, "repo", "14 quick dismissals · replayed on 61 uses · under your rule"),
            ("cite", "STEP SPEC", "Research steps cite the vendor’s docs before blog posts", 0, "studio", "1 studio session · widens after it helps again"),
        ];
        v_flex()
            .flex_1()
            .h_full()
            .p_6()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(div().text_lg().font_semibold().child("What it learned this week"))
                    .child(Button::new("since").outline().small().icon(Lucide::Undo2).label(if self.rolled_since { "Rolled back to Monday" } else { "Roll back to Monday" }).on_click(cx.listener(|this, _, _, cx| { this.rolled_since = true; cx.notify(); }))),
            )
            .children(items.into_iter().map(|(id, kind, what, scope, repo, from)| {
                let back = self.rolled_since || self.rolled_back_items.contains(id);
                v_flex()
                    .gap_1p5()
                    .p_3()
                    .rounded(px(12.))
                    .border_1()
                    .border_color(pal.border)
                    .when(back, |d| d.opacity(0.6))
                    .child(h_flex().gap_2().child(div().text_xs().font_semibold().text_color(pal.muted_fg).child(kind)).child(div().text_sm().child(what)))
                    .child(
                        h_flex()
                            .gap_3()
                            .items_center()
                            .child(scope_ladder(scope, repo, pal))
                            .child(div().text_xs().text_color(pal.muted_fg).child(from))
                            .child(div().flex_1())
                            .child(if back {
                                div().text_xs().text_color(pal.muted_fg).child("rolled back by you").into_any_element()
                            } else {
                                Button::new(SharedString::from(format!("rb-{id}"))).ghost().xsmall().icon(Lucide::Undo2).label("Roll back").on_click(cx.listener(move |this, _, _, cx| { this.rolled_back_items.insert(id); cx.notify(); })).into_any_element()
                            }),
                    )
            }))
            .child(div().text_xs().text_color(pal.muted_fg).child("Local first: a lesson applies to its session, widens to its repo after it helps again with a clean replay, and to all repos only with your OK. Learning never changes your checks, your decisions or DoneRight’s four rules."))
    }

    // ---------- numbers: plan windows, spend, infra ----------
    pub(crate) fn numbers_view(&self, pal: &Pal) -> impl IntoElement {
        let bar = |label: &'static str, used: f32, note: &'static str, id: &'static str| {
            v_flex()
                .gap_1()
                .child(h_flex().justify_between().child(div().text_sm().child(label)).child(div().text_xs().text_color(pal.muted_fg).child(note)))
                .child(Progress::new(id).value(used).color(if used >= 70. { c_back() } else { c_run() }))
        };
        v_flex()
            .flex_1()
            .h_full()
            .p_6()
            .gap_3()
            .child(div().text_lg().font_semibold().child("Numbers"))
            .child(
                h_flex()
                    .gap_3()
                    .items_start()
                    .child(
                        box_(pal)
                            .w(px(420.))
                            .child(div().font_semibold().child("Your plans"))
                            .child(bar("Claude Code · Max · 5-hour window", 38., "38% · resets 1:40 pm", "p1"))
                            .child(bar("Claude Code · Max · this week", 41., "41% · resets Monday", "p2"))
                            .child(bar("Codex · Pro · 5-hour window", 9., "9% · resets 2:55 pm", "p3"))
                            .child(div().text_xs().text_color(pal.muted_fg).child("DoneRight’s own work keeps out of the last 30% of each window, so it stays yours.")),
                    )
                    .child(
                        box_(pal)
                            .w(px(420.))
                            .child(div().font_semibold().child("This week"))
                            .children([
                                ("“Is it done?” asked by you", "0", "was 14 a week"),
                                ("False “done” sent back", "6", "none reached you"),
                                ("Risky commands stopped", "4", "kill by port ×3, paid CI ×1"),
                                ("Building and improving", "4%", "of one 5-hour window"),
                                ("Hindsight trial", "$0.38", "its own model calls"),
                            ].into_iter().map(|(k, v, n)| {
                                h_flex().gap_3().py_1().border_b_1().border_color(pal.border.opacity(0.6)).child(div().flex_1().text_sm().child(k)).child(div().text_sm().font_semibold().child(v)).child(div().w(px(190.)).text_xs().text_color(pal.muted_fg).child(n))
                            })),
                    ),
            )
            .child(
                h_flex()
                    .gap_3()
                    .items_start()
                    .child(
                        box_(pal)
                            .w(px(420.))
                            .child(h_flex().justify_between().child(div().font_semibold().child("Sent back before it reached you")).child(div().text_xs().text_color(pal.muted_fg).child("6 this week")))
                            .child(
                                div().h(px(150.)).w_full().child(
                                    BarChart::new(vec![("Mon", 1.), ("Tue", 0.), ("Wed", 2.), ("Thu", 1.), ("Fri", 0.), ("Sat", 1.), ("Sun", 1.)])
                                        .band(|d: &(&'static str, f64)| d.0)
                                        .value(|d| d.1)
                                        .fill(|_, _, _, _| c_back())
                                        .tooltip_value(|_, v| SharedString::from(format!("{v} sent back"))),
                                ),
                            )
                            .child(div().text_xs().text_color(pal.muted_fg).child("Each one failed a check and went back to its agent, so you never saw it.")),
                    )
                    .child(
                        box_(pal)
                            .w(px(420.))
                            .child(div().font_semibold().child("Infra"))
                            .children([
                                ("web · Vercel", St::Ok, "$20/mo"),
                                ("api · Fly", St::Ok, "$31/mo"),
                                ("db · Postgres", St::Ok, "$25/mo"),
                                ("staging-old · stopped by your guard", St::Back, "saves about $45/mo"),
                                ("Stripe · test mode", St::Ok, "$0"),
                            ].into_iter().map(|(n, s, c)| {
                                h_flex().gap_2().items_center().py_1().border_b_1().border_color(pal.border.opacity(0.6)).child(div().size(px(8.)).rounded_full().bg(pal.st(s))).child(div().flex_1().text_sm().child(n)).child(div().text_xs().text_color(pal.muted_fg).child(c))
                            })),
                    ),
            )
    }

    // ---------- every ask that needs you ----------
    pub(crate) fn needs_full(&self, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        let waiting = self.waiting();
        let asks: [(&'static str, &'static str, &'static str, &'static str, [&'static str; 2]); 3] = [
            ("signin", "UNBLOCK · CODEX APP · #131", "Renew shopper-2’s sign-in", "The receipt journey needs it, and it takes your 2FA, so only you can do it.", ["Open", "Later"]),
            ("pay", "APPROVAL · CODEX APP · #133", "Run the live payment workflow? About $0.40", "It’s past #133’s budget for live runs. Everything else in the check already passed.", ["Run it", "Not now"]),
            ("send", "NEW ACCESS · SCREENSHOT-LINE", "Let screenshot-line send screenshots to a Claude or Codex chat?", "It proposed this from how you use it. Replayed on your 61 past uses with no other change.", ["Allow it", "Not now"]),
        ];
        let open: Vec<_> = asks.iter().filter(|a| !self.answers.contains_key(a.0)).collect();
        let n = open.len() + usize::from(waiting) + usize::from(self.memory.is_none());
        v_flex()
            .flex_1()
            .h_full()
            .p_6()
            .gap_3()
            .child(div().text_lg().font_semibold().child(if n == 0 { "Nothing needs you".to_string() } else if n == 1 { "One thing needs you".to_string() } else { format!("{n} things need you. Answer them in one visit.") }))
            .when(waiting, |d| {
                d.child(card(pal, c_you(), "TASTE CALL · CLAUDE APP · #123", "Does the new checkout layout look right?", "5 of 6 checks passed; screens needs your eye. Look at: the total and the Pay button stay in view after the move. Your past calls: 3 moves like this approved, 1 put back.")
                    .child(h_flex().gap_2().child(shot("Before", false, pal)).child(shot("After", true, pal)).max_w(px(360.)))
                    .child(h_flex().gap_2().child(Button::new("nl").primary().small().label("Looks right").on_click(cx.listener(|this, _, window, cx| this.answer_taste("Looks right", window, cx)))).child(Button::new("nb").outline().small().label("Put it back").on_click(cx.listener(|this, _, window, cx| this.answer_taste("Put it back", window, cx))))))
            })
            .when(self.memory.is_none(), |d| d.child(self.memory_trial(pal, cx).max_w(px(620.))))
            .children(open.into_iter().map(|(key, kind, title, body, opts)| {
                let k = *key;
                let (a, b) = (opts[0], opts[1]);
                card(pal, c_you(), kind, title, body).child(
                    h_flex()
                        .gap_2()
                        .child(Button::new(SharedString::from(format!("a-{k}"))).primary().small().label(a).on_click(cx.listener(move |this, _, window, cx| {
                            this.answers.insert(k, a);
                            toast(window, cx, "Answered", format!("“{a}.” The agent that asked hears it at its next stop."));
                            cx.notify();
                        })))
                        .child(Button::new(SharedString::from(format!("b-{k}"))).outline().small().label(b).on_click(cx.listener(move |this, _, _, cx| { this.answers.insert(k, b); cx.notify(); }))),
                )
            }))
            .when(self.proposal == Proposal::Pending, |d| d.child(section("SUGGESTED · NOT COUNTED AS AN ASK", pal)).child(self.proposal_card(pal, cx)))
            .when(!self.answers.is_empty(), |d| {
                d.child(section("ANSWERED TODAY", pal)).children(self.answers.iter().map(|(k, v)| {
                    h_flex().gap_3().child(div().text_sm().child(match *k { "signin" => "Renew shopper-2’s sign-in", "pay" => "Run the live payment workflow", _ => "Let screenshot-line send to a chat" })).child(div().text_xs().text_color(pal.muted_fg).child(*v))
                }))
            })
            .child(div().text_xs().text_color(pal.muted_fg).child("Handled without you today: 6 false “done” sent back, 4 risky commands stopped, 2 questions answered from your decisions."))
    }
}

impl DoneRight {
    pub(crate) fn playbook_details(&self, pal: &Pal) -> impl IntoElement {
        let steps = [
            ("Research", "every claim cites a source that resolves", St::Ok, "14 sources resolve"),
            ("Plan and issues", "acceptance criteria and a proof line per issue", St::Ok, "approved by you · 8 issues in 3 lanes"),
            ("Build in parallel", "each lane in its own worktree, within each plan’s window", St::Run, "Lane B’s #4 went back once"),
            ("Prove", "every “done” passes the done gate", St::Run, "3 of 8 proven"),
            ("Merge", "a green status on the exact commit, in a safe order", St::Idle, "dry merges set the order"),
            ("Release", "the same journeys pass on the live site", St::Idle, "you approve it"),
        ];
        v_flex()
            .w(px(320.))
            .h_full()
            .flex_shrink_0()
            .border_l_1()
            .border_color(pal.border)
            .bg(pal.card)
            .p_4()
            .gap_2()
            .child(h_flex().gap_2().items_center().child(Tag::primary().small().child("studio")).child(div().text_sm().font_semibold().child("Class bookings for a yoga studio")))
            .child(div().text_xs().text_color(pal.muted_fg).child("dr create · your usual steps, drafted from 31 past projects"))
            .children(steps.into_iter().map(|(name, gate, st, note)| {
                h_flex()
                    .gap_2()
                    .items_start()
                    .py_1p5()
                    .border_b_1()
                    .border_color(pal.border.opacity(0.6))
                    .child(div().mt_1().size(px(8.)).flex_shrink_0().rounded_full().bg(pal.st(st)))
                    .child(v_flex().flex_1().min_w_0().child(div().text_sm().font_semibold().child(name)).child(div().text_xs().text_color(pal.muted_fg).child(gate)).child(div().text_xs().child(note)))
            }))
            .child(section("PLANS", pal))
            .child(div().text_xs().text_color(pal.muted_fg).child("Claude Code · Max is at 72% of its 5-hour window, past the 70% line, so lane C waits for the 1:40 pm reset. Codex has room."))
    }

    pub(crate) fn extension_details(&self, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        let decided = self.answers.get("send").copied();
        v_flex()
            .w(px(320.))
            .h_full()
            .flex_shrink_0()
            .border_l_1()
            .border_color(pal.border)
            .bg(pal.card)
            .p_4()
            .gap_2()
            .child(h_flex().gap_2().items_center().child(Tag::primary().small().child("0.1.1")).child(div().text_sm().font_semibold().child("screenshot-line")))
            .child(div().text_xs().text_color(pal.muted_fg).child("“Hang every screenshot I take on a line under the menu bar.” Built by Claude Code on your plan: 3% of a 5-hour window."))
            .child(section("IT CAN", pal))
            .children(["see new files in ~/Desktop/Screenshots", "show a line under the menu bar", "copy a screenshot to the clipboard"].into_iter().map(|t| div().text_xs().child(format!("✓ {t}"))))
            .child(section("IT CAN’T", pal))
            .children(["use the network", "move, change or delete files", "read any other folder"].into_iter().map(|t| div().text_xs().text_color(pal.muted_fg).child(format!("✗ {t}"))))
            .child(section("IMPROVED WITH USE", pal))
            .child(div().text_xs().child("0.1.1 waits 300 ms before the line glides down. Applied on its own under the rule you approved after 3 changes like it."))
            .child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(c_you())
                    .child(div().text_xs().font_semibold().text_color(c_you()).child("NEW ACCESS"))
                    .child(div().text_sm().child("Let it send screenshots to a Claude or Codex chat?"))
                    .child(match decided {
                        Some(a) => div().text_xs().text_color(pal.muted_fg).child(format!("You answered: {a}")).into_any_element(),
                        None => h_flex()
                            .gap_2()
                            .child(Button::new("x-allow").primary().small().label("Allow it").on_click(cx.listener(|this, _, _, cx| { this.answers.insert("send", "Allow it"); cx.notify(); })))
                            .child(Button::new("x-no").outline().small().label("Not now").on_click(cx.listener(|this, _, _, cx| { this.answers.insert("send", "Not now"); cx.notify(); })))
                            .into_any_element(),
                    }),
            )
    }
}
