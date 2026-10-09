//! What you and your agents added to the app, or changed in it: gadgets on the home screen,
//! items in the sidebar, a tab in a claim's details, or your version of part of a shipped
//! view. Each was built in one shot from one sentence, yours or DoneRight's suggestion from
//! how you work, and is drawn with the app's own components, so it looks native and sits
//! where you put it, among the shipped pieces. "Show what changed" outlines every one of
//! them; a click on the outline says where it came from.

use super::*;
use gpui_kit::component::{
    WindowExt as _,
    command::{Command, CommandGroup, CommandItem},
    description_list::DescriptionList,
    notification::Notification,
};

/// The one color for "added or changed by you or your agent". It is not a status, so it is drawn dashed.
pub(crate) fn c_added() -> Hsla { hsla(188. / 360., 0.80, 0.36, 1.) }

pub(crate) const GADGET_H: f32 = 124.;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Kind { Screens, Inbox, Bounces, Buddy, Receipt, Deploy, Other }

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Proposal { Pending, Accepted, Dismissed }

#[derive(Clone)]
pub(crate) struct Added {
    pub key: SharedString,
    pub version: &'static str,
    pub words: SharedString,
    pub suggested: bool,
    pub agent: &'static str,
    pub cost: &'static str,
    pub when: &'static str,
    pub kind: Kind,
    pub nav: Option<(SharedString, Lucide)>,
    pub gadget: bool,
    pub tab: bool,
    /// a shipped view this changes; the shipped one stays one click away
    pub replaces: Option<View>,
    pub can: Vec<&'static str>,
    pub cant: Vec<&'static str>,
    pub history: Vec<(&'static str, &'static str)>,
    pub age: f32,
    pub removed: bool,
}

impl Added {
    pub(crate) fn places(&self) -> String {
        let mut v = vec![];
        if let Some((label, _)) = &self.nav { v.push(format!("sidebar ({label})")); }
        if self.gadget { v.push("home".to_string()); }
        if self.tab { v.push("claim details".to_string()); }
        if self.replaces.is_some() { v.push("Flow, in place of the part it changes".to_string()); }
        if v.is_empty() { "over your screen, not in this app".to_string() } else { v.join(" · ") }
    }
}

pub(crate) fn seed_added() -> Vec<Added> {
    vec![
        Added {
            key: "screenshot-line".into(),
            version: "0.1.1",
            words: "Hang every screenshot I take on a line under the menu bar.".into(),
            suggested: false,
            agent: "Claude Code",
            cost: "3% of one 5-hour window",
            when: "Wednesday",
            kind: Kind::Screens,
            nav: Some(("Screenshots".into(), Lucide::Images)),
            gadget: true,
            tab: false,
            replaces: None,
            can: vec!["see new files in ~/Desktop/Screenshots", "show a line under the menu bar", "copy a screenshot to the clipboard"],
            cant: vec!["use the network", "move, change or delete files", "read any other folder"],
            history: vec![("0.1.0", "built from your words, Wednesday"), ("0.1.1", "waits 300 ms before the line glides down; applied under your rule")],
            age: 1e6,
            removed: false,
        },
        Added {
            key: "receipt-mail-watch".into(),
            version: "0.3.1",
            words: "Watch the receipts mailbox and tell me when something looks off.".into(),
            suggested: false,
            agent: "Codex",
            cost: "2% of one 5-hour window",
            when: "Oct 2",
            kind: Kind::Inbox,
            nav: None,
            gadget: true,
            tab: false,
            replaces: None,
            can: vec!["read the receipts mailbox", "show a gadget on your home"],
            cant: vec!["send, move or delete mail", "read any other mailbox"],
            history: vec![("0.3.0", "built from your words"), ("0.3.1", "groups bounces by sender; applied on its own Oct 2")],
            age: 1e6,
            removed: false,
        },
        Added {
            key: "receipt-diff".into(),
            version: "0.1.0",
            words: "You opened the receipt on 11 of the last 12 checked claims, so DoneRight suggested putting it next to the checks.".into(),
            suggested: true,
            agent: "Claude Code",
            cost: "1% of one 5-hour window",
            when: "Monday",
            kind: Kind::Receipt,
            nav: None,
            gadget: false,
            tab: true,
            replaces: None,
            can: vec!["read a claim’s journey screenshots and receipts", "add a tab to a claim’s details"],
            cant: vec!["use the network", "change a check or its verdict"],
            history: vec![("0.1.0", "suggested Monday; you said yes")],
            age: 1e6,
            removed: false,
        },
        Added {
            key: "cursor-buddy".into(),
            version: "0.1.0",
            words: "Point at things on my screen when I hold ⌥Space.".into(),
            suggested: false,
            agent: "Claude Code",
            cost: "4% of one 5-hour window",
            when: "Thursday",
            kind: Kind::Buddy,
            nav: None,
            gadget: false,
            tab: false,
            replaces: None,
            can: vec!["see your screen only while you hold ⌥Space", "draw a pointer over the screen"],
            cant: vec!["save what it sees without asking", "use the network"],
            history: vec![("0.1.0", "built from your words, Thursday")],
            age: 1e6,
            removed: false,
        },
        Added {
            key: "flow-also".into(),
            version: "0.1.0",
            words: "In Flow, also show the other claims that are moving, under Now.".into(),
            suggested: false,
            agent: "Claude Code",
            cost: "1% of one 5-hour window",
            when: "Tuesday",
            kind: Kind::Other,
            nav: None,
            gadget: false,
            tab: false,
            replaces: Some(View::Flow),
            can: vec!["read the claims you can already see", "change the Flow view"],
            cant: vec!["use the network", "change a claim or its verdict"],
            history: vec![("0.1.0", "your version of the shipped Flow view; the shipped one is one click away")],
            age: 1e6,
            removed: false,
        },
    ]
}

pub(crate) fn deploy_gadget() -> Added {
    Added {
        key: "deploy-status".into(),
        version: "0.1.0",
        words: "You asked “did it deploy?” 9 times this week, in 4 sessions.".into(),
        suggested: true,
        agent: "Claude Code",
        cost: "1% of one 5-hour window",
        when: "just now",
        kind: Kind::Deploy,
        nav: None,
        gadget: true,
        tab: false,
        replaces: None,
        can: vec!["read deploy status with your Vercel and Fly logins, read-only"],
        cant: vec!["deploy, roll back or change anything", "use any other login"],
        history: vec![("0.1.0", "built in a copy, checks passed; you added it")],
        age: 0.,
        removed: false,
    }
}

/// What an installed build adds to the app, read from its words.
pub(crate) fn added_from_build(words: &str) -> Added {
    let w = words.to_lowercase();
    let (kind, label, icon, key): (Kind, SharedString, Lucide, SharedString) = if w.contains("screenshot") {
        (Kind::Screens, "Screenshot line".into(), Lucide::Images, "screenshot-line-2".into())
    } else if w.contains("receipt") || w.contains("bounce") || w.contains("inbox") {
        (Kind::Bounces, "Bounces".into(), Lucide::MailWarning, "bounce-watch".into())
    } else if w.contains("point") || w.contains('⌥') {
        (Kind::Buddy, "Cursor buddy".into(), Lucide::MousePointer2, "cursor-buddy-2".into())
    } else {
        let first: Vec<String> = words.split_whitespace().take(3).map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase()).filter(|s| !s.is_empty()).collect();
        let mut label = first.join(" ");
        if let Some(c) = label.get(0..1) {
            label = c.to_uppercase() + &label[1..];
        }
        (Kind::Other, label.into(), Lucide::Sparkles, first.join("-").into())
    };
    Added {
        key,
        version: "0.1.0",
        words: words.to_string().into(),
        suggested: false,
        agent: "Claude Code",
        cost: "2% of one 5-hour window",
        when: "just now",
        kind,
        nav: Some((label, icon)),
        gadget: false,
        tab: false,
        replaces: None,
        can: vec!["read only what its grant declares", "show a panel in your sidebar"],
        cant: vec!["send anything", "change files outside its own folder"],
        history: vec![("0.1.0", "built from your words, checked; you added it")],
        age: 0.,
        removed: false,
    }
}

/// The small dashed chip on a sidebar item you or your agents added or changed.
pub(crate) fn added_chip(t: &'static str) -> AnyElement {
    div().px_1().rounded(px(4.)).border_1().border_dashed().border_color(c_added()).text_xs().text_color(c_added()).child(t).into_any_element()
}

pub(crate) fn toast(window: &mut Window, cx: &mut App, title: impl Into<SharedString>, msg: impl Into<SharedString>) {
    let hold = std::env::var("DR_HOLD").is_ok();
    window.push_notification(Notification::success(msg.into()).title(title).autohide(!hold), cx);
}

const IDEAS: [&str; 3] = ["A gadget that shows whether each repo deployed", "Hang every screenshot on a line under the menu bar", "Watch the receipts inbox for bounces"];
const GO: [(&str, Lucide, View); 8] = [
    ("Flow", Lucide::Workflow, View::Flow),
    ("Needs you", Lucide::Inbox, View::NeedsYou),
    ("Sessions", Lucide::LayoutDashboard, View::Sessions),
    ("Map", Lucide::Map, View::Map),
    ("Numbers", Lucide::Gauge, View::Numbers),
    ("Add anything", Lucide::Sparkles, View::Add),
    ("Learned", Lucide::Brain, View::Learned),
    ("Wiki", Lucide::BookOpen, View::Wiki),
];

pub(crate) fn deploy_preview(pal: &Pal) -> Div {
    let row = |name: &'static str, st: St, note: &'static str| {
        h_flex()
            .gap_1p5()
            .items_center()
            .child(div().size(px(7.)).flex_shrink_0().rounded_full().bg(pal.st(st)))
            .child(div().w(px(34.)).text_xs().font_semibold().child(name))
            .child(div().text_xs().text_color(pal.muted_fg).child(note))
    };
    v_flex().gap_0p5().child(row("shop", St::Ok, "v1.42 · 12 min ago")).child(row("api", St::Run, "deploying · 1m 20s")).child(row("docs", St::Ok, "yesterday"))
}

impl DoneRight {
    pub(crate) fn idx_of(&self, kind: Kind) -> Option<usize> {
        self.added.iter().position(|a| a.kind == kind && !a.removed)
    }

    /// What you or your agents added or changed. Plain normally, with a brief glow when it
    /// has just appeared; with "Show what changed" on, a dashed outline and a chip naming its source.
    pub(crate) fn mark(&self, idx: usize, inner: impl IntoElement, cx: &mut Context<Self>) -> AnyElement {
        let a = &self.added[idx];
        let fresh = a.age < 5.;
        if !self.show_added && !fresh {
            return inner.into_any_element();
        }
        let chip = if a.replaces.is_some() { format!("✦ your version · {}", a.key) } else if a.suggested { format!("✦ one shot · suggested · {}", a.key) } else { format!("✦ one shot · {}", a.key) };
        let words = a.words.clone();
        let glow = if fresh && !self.show_added { (1. - a.age / 5.).clamp(0., 1.) } else { 1. };
        div()
            .relative()
            .child(inner)
            .child(
                div()
                    .absolute()
                    .top(px(-5.))
                    .left(px(-5.))
                    .right(px(-5.))
                    .bottom(px(-5.))
                    .rounded(px(16.))
                    .border_1()
                    .when(self.show_added, |d| d.border_dashed())
                    .border_color(c_added().opacity(glow))
                    .bg(c_added().opacity(0.05 * glow)),
            )
            .when(self.show_added, |d| {
                d.child(
                    div()
                        .id(("mark", idx))
                        .absolute()
                        .top(px(-14.))
                        .left(px(10.))
                        .px_1p5()
                        .rounded(px(6.))
                        .bg(c_added())
                        .text_color(white())
                        .text_xs()
                        .cursor_pointer()
                        .child(chip)
                        .tooltip(move |window, cx| Tooltip::new(format!("“{words}” · click for where it came from")).build(window, cx))
                        .on_click(cx.listener(move |this, _, window, cx| this.open_about(idx, window, cx)))
                        .test_support(),
                )
            })
            .into_any_element()
    }

    // ---------- the home strip: gadgets your agents added, and one suggestion ----------
    pub(crate) fn gadget_strip(&self, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        let ids: Vec<usize> = self.added.iter().enumerate().filter(|(_, a)| a.gadget && !a.removed).map(|(i, _)| i).collect();
        h_flex()
            .gap_3()
            .px_5()
            .pt_5()
            .items_start()
            .children(ids.into_iter().map(|i| self.gadget(i, pal, cx)))
            .when(self.proposal == Proposal::Pending, |d| d.child(self.proposal_tile(pal, cx)))
            .child(
                div()
                    .id("add-gadget")
                    .w(px(112.))
                    .h(px(GADGET_H))
                    .flex_shrink_0()
                    .rounded(px(12.))
                    .border_1()
                    .border_dashed()
                    .border_color(pal.border)
                    .cursor_pointer()
                    .hover(|d| d.bg(pal.muted))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .child(Icon::from(Lucide::Plus).size_4().text_color(pal.muted_fg))
                    .child(div().text_xs().text_color(pal.muted_fg).child("Add a gadget"))
                    .child(div().text_xs().text_color(pal.muted_fg.opacity(0.8)).child("say it in words"))
                    .on_click(cx.listener(|this, _, window, cx| this.open_palette(Some("A gadget that shows "), window, cx)))
                    .test_support(),
            )
    }

    fn gadget(&self, idx: usize, pal: &Pal, cx: &mut Context<Self>) -> AnyElement {
        let a = &self.added[idx];
        let (icon, title) = match a.kind {
            Kind::Screens => (Lucide::Images, "Screenshots"),
            Kind::Inbox => (Lucide::Mail, "Receipts inbox"),
            Kind::Deploy => (Lucide::Rocket, "Deploys"),
            _ => (Lucide::Sparkles, "Gadget"),
        };
        let body = match a.kind {
            Kind::Screens => v_flex()
                .child(h_flex().gap_1p5().items_end().child(div().text_2xl().font_semibold().child("6")).child(div().pb_1().text_xs().text_color(pal.muted_fg).child("today")))
                .child(div().text_xs().text_color(pal.muted_fg).child("last one 2 min ago, on your line")),
            Kind::Inbox => v_flex()
                .child(h_flex().gap_1p5().items_end().child(div().text_2xl().font_semibold().child("2")).child(div().pb_1().text_xs().text_color(pal.muted_fg).child("bounces")))
                .child(div().text_xs().text_color(pal.muted_fg).child("newest 9:12 · receipt for #1042")),
            Kind::Deploy => deploy_preview(pal),
            _ => v_flex(),
        };
        let has_panel = a.nav.is_some();
        let card = v_flex()
            .id(("gadget", idx))
            .w(px(206.))
            .h(px(GADGET_H))
            .flex_shrink_0()
            .p_3()
            .gap_1p5()
            .rounded(px(12.))
            .border_1()
            .border_color(pal.border)
            .bg(pal.card)
            .cursor_pointer()
            .hover(|d| d.border_color(pal.muted_fg))
            .child(
                h_flex()
                    .gap_1p5()
                    .items_center()
                    .child(Icon::from(icon).size_3p5().text_color(pal.muted_fg))
                    .child(div().flex_1().text_xs().font_semibold().text_color(pal.muted_fg).child(title)),
            )
            .child(body)
            .on_click(cx.listener(move |this, _, window, cx| {
                if has_panel {
                    this.view = View::Ext(idx);
                    cx.notify();
                } else {
                    this.open_about(idx, window, cx);
                }
            }))
            .test_support();
        self.mark(idx, card, cx)
    }

    fn proposal_tile(&self, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w(px(256.))
            .h(px(GADGET_H))
            .flex_shrink_0()
            .p_3()
            .gap_1p5()
            .rounded(px(12.))
            .border_1()
            .border_dashed()
            .border_color(c_added())
            .bg(c_added().opacity(0.05))
            .child(h_flex().gap_1p5().items_center().child(Icon::from(Lucide::Sparkles).size_3p5().text_color(c_added())).child(div().text_xs().font_semibold().text_color(c_added()).child("Suggested gadget · Deploys")))
            .child(div().text_xs().text_color(pal.muted_fg).child("You asked “did it deploy?” 9 times this week. Built in a copy; its checks passed."))
            .child(div().flex_1())
            .child(
                h_flex()
                    .gap_1()
                    .child(Button::new("p-add").primary().xsmall().label("Add it").on_click(cx.listener(|this, _, window, cx| this.accept_proposal(window, cx))))
                    .child(Button::new("p-see").ghost().xsmall().label("Preview").on_click(cx.listener(|this, _, _, cx| { this.view = View::Flow; this.flow_sel = 3; cx.notify(); })))
                    .child(Button::new("p-no").ghost().xsmall().label("Not now").on_click(cx.listener(|this, _, window, cx| this.dismiss_proposal(window, cx)))),
            )
    }

    pub(crate) fn accept_proposal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.proposal != Proposal::Pending {
            return;
        }
        self.proposal = Proposal::Accepted;
        self.added.push(deploy_gadget());
        toast(window, cx, "Added to your home", "Deploys is on your home screen. Claude Code built it on your Max plan: 1% of one window.");
        cx.notify();
    }

    pub(crate) fn dismiss_proposal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.proposal != Proposal::Pending {
            return;
        }
        self.proposal = Proposal::Dismissed;
        toast(window, cx, "Not now", "It won’t suggest a deploy gadget again this month. Ask for it any time.");
        cx.notify();
    }

    pub(crate) fn propose_toast(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let me = cx.entity().downgrade();
        window.push_notification(
            Notification::new()
                .icon(Icon::from(Lucide::Sparkles).text_color(c_added()))
                .title("A gadget for your deploys?")
                .message("You asked “did it deploy?” 9 times this week. Your agent built one in a copy, and it passed its checks.")
                .action(move |_, _, cx| {
                    let me = me.clone();
                    Button::new("p-show").primary().xsmall().label("Show me").on_click(cx.listener(move |n, _, window, cx| {
                        n.dismiss(window, cx);
                        let _ = me.update(cx, |this, cx| {
                            this.view = View::Flow;
                            this.flow_sel = 3;
                            cx.notify();
                        });
                    }))
                }),
            cx,
        );
    }

    /// A build your agent finished: installing it puts its panel in the sidebar.
    pub(crate) fn install_build(&mut self, i: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(b) = self.builds.get_mut(i) else { return };
        if b.installed {
            return;
        }
        b.installed = true;
        let a = added_from_build(&b.words);
        let label = a.nav.as_ref().map(|n| n.0.to_string()).unwrap_or_default();
        self.added.push(a);
        b.added = Some(self.added.len() - 1);
        toast(window, cx, "Added to Work", format!("“{label}” sits in Work, next to Sessions. Built in one shot from your words; remove it any time."));
        cx.notify();
    }

    // ---------- inside a claim: the receipt tab DoneRight suggested ----------
    pub(crate) fn receipt_diff(&self, s: &Snap, pal: &Pal) -> impl IntoElement {
        use Phase::*;
        let failing = matches!(s.phase, Back | Agent2 | ToClaim2 | Claim2 | ToChecks2);
        let before = matches!(s.phase, Ticket | ToAgent | Agent1 | ToClaim1 | Claim1 | ToChecks1 | Checks1);
        let rows = [("Subtotal", "$20.01", "$20.01", true), ("Coupon 10%", "−$2.00", "−$2.00", true), ("Total", "$18.01", if failing { "$18.00" } else { "$18.01" }, !failing)];
        v_flex()
            .gap_1()
            .p_2()
            .child(
                h_flex()
                    .child(div().flex_1())
                    .child(div().w(px(70.)).text_xs().text_color(pal.muted_fg).child("Cart"))
                    .child(div().w(px(70.)).text_xs().text_color(pal.muted_fg).child("Receipt")),
            )
            .children(rows.into_iter().map(|(k, a, b, ok)| {
                h_flex()
                    .py_1()
                    .border_b_1()
                    .border_color(pal.border.opacity(0.6))
                    .child(div().flex_1().text_sm().child(k))
                    .child(div().w(px(70.)).text_sm().child(a))
                    .child(div().w(px(70.)).text_sm().font_semibold().text_color(if ok { pal.fg } else { c_back() }).child(b))
            }))
            .child(div().pt_1().text_xs().text_color(pal.muted_fg).child(if before {
                "Fills in when the first journey run finishes."
            } else if failing {
                "Journey run 3 of 29: the receipt is 1¢ short."
            } else {
                "29 of 29 runs: cart and receipt match."
            }))
    }

    // ---------- a panel in the sidebar, built from your words ----------
    pub(crate) fn ext_view(&self, idx: usize, pal: &Pal, cx: &mut Context<Self>) -> impl IntoElement {
        let a = self.added[idx].clone();
        let label = a.nav.as_ref().map(|n| n.0.clone()).unwrap_or(a.key.clone());
        let body: AnyElement = match a.kind {
            Kind::Screens => self.screens_panel(pal).into_any_element(),
            Kind::Bounces | Kind::Inbox => bounces_panel(pal).into_any_element(),
            Kind::Buddy => buddy_panel(pal).into_any_element(),
            _ => other_panel(pal).into_any_element(),
        };
        v_flex()
            .flex_1()
            .h_full()
            .p_6()
            .gap_4()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(div().text_lg().font_semibold().child(label))
                    .child(Tag::secondary().small().child(format!("{} {}", a.key, a.version)))
                    .child(div().flex_1())
                    .child(Button::new("about").ghost().small().icon(Lucide::Info).label("Where it came from").on_click(cx.listener(move |this, _, window, cx| this.open_about(idx, window, cx)))),
            )
            .child(div().text_xs().text_color(pal.muted_fg).child(format!("From “{}” · {} built it on your plan", a.words, a.agent)))
            .child(self.mark(idx, body, cx))
    }

    fn screens_panel(&self, pal: &Pal) -> impl IntoElement {
        let allowed = self.answers.get("send") == Some(&"Allow it");
        let times = ["2 min ago", "9:41 am", "9:12 am", "8:55 am", "8:20 am", "Yesterday", "Yesterday", "Monday"];
        h_flex().flex_wrap().gap_3().p_1().children(times.into_iter().enumerate().map(|(i, t)| {
            let dark = i % 3 == 1;
            let shot = v_flex()
                .h(px(116.))
                .p_2()
                .gap_1()
                .rounded(px(8.))
                .border_1()
                .border_color(pal.border)
                .bg(if dark { hsla(240. / 360., 0.10, 0.12, 1.) } else { pal.bg })
                .child(h_flex().gap_1().child(div().size(px(6.)).rounded_full().bg(c_back().opacity(0.7))).child(div().size(px(6.)).rounded_full().bg(c_you().opacity(0.4))).child(div().size(px(6.)).rounded_full().bg(c_ok().opacity(0.7))))
                .children((0..4).map(move |j| {
                    let w = [150., 96., 128., 70.][(i + j) % 4];
                    div().h(px(7.)).w(px(w)).rounded(px(2.)).bg(if dark { c_ok().opacity(0.35) } else if j == 1 && i % 3 == 2 { c_run().opacity(0.35) } else { pal.muted })
                }));
            v_flex()
                .w(px(196.))
                .gap_1()
                .child(shot)
                .child(
                    h_flex()
                        .items_center()
                        .gap_1()
                        .child(div().flex_1().text_xs().text_color(pal.muted_fg).child(t))
                        .child(Button::new(("copy", i)).ghost().xsmall().label("Copy"))
                        .child(Button::new(("send", i)).ghost().xsmall().label("Send to chat").disabled(!allowed).tooltip(if allowed { "Send to a Claude or Codex chat" } else { "New access: waits for your OK in Needs you" })),
                )
        }))
    }

    // ---------- where an added piece came from ----------
    pub(crate) fn open_about(&mut self, idx: usize, window: &mut Window, cx: &mut Context<Self>) {
        let a = self.added[idx].clone();
        let me = cx.entity().downgrade();
        window.open_sheet(cx, move |sheet, _, cx| {
            let pal = Pal::of(cx);
            let a = a.clone();
            let me = me.clone();
            let key = a.key.clone();
            let prev = (a.history.len() > 1).then(|| a.history[a.history.len() - 2].0);
            sheet
                .size(px(460.))
                .title(div().child(format!("{} {}", a.key, a.version)))
                .child(about_body(&a, &pal))
                .footer(
                    h_flex()
                        .gap_2()
                        .when_some(prev, |d, v| {
                            let key = key.clone();
                            d.child(Button::new("rb").outline().small().icon(Lucide::Undo2).label(format!("Roll back to {v}")).on_click(move |_, window, cx| {
                                window.close_sheet(cx);
                                toast(window, cx, "Rolled back", format!("{key} is back on {v}. It stays there unless you say otherwise."));
                            }))
                        })
                        .child(Button::new("rm").ghost().small().icon(Lucide::Trash).label("Remove").on_click(move |_, window, cx| {
                            let _ = me.update(cx, |this, cx| {
                                this.added[idx].removed = true;
                                if this.view == View::Ext(idx) {
                                    this.view = View::Flow;
                                }
                                cx.notify();
                            });
                            window.close_sheet(cx);
                            toast(window, cx, "Removed", format!("{key} is off your Mac. Say the word to bring it back."));
                        })),
                )
        });
    }

    // ---------- ⌘K: say what you want, or where to go ----------
    pub(crate) fn open_palette(&mut self, query: Option<&str>, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.palette.clone();
        let q0 = query.unwrap_or("").to_string();
        state.update(cx, |s, cx| s.set_query(q0, window, cx));
        let me = cx.entity().downgrade();
        let st = state.clone();
        let show_added = self.show_added;
        window.open_dialog(cx, move |dialog, _, cx| {
            let q = st.read(cx).query(cx).trim().to_string();
            let me = me.clone();
            let typed = q.clone();
            let mut ask = CommandGroup::new().label("Ask your agent to build");
            if !q.is_empty() {
                ask = ask.item(CommandItem::new().label(format!("“{q}”")).icon(Lucide::Sparkles));
            }
            for idea in IDEAS {
                ask = ask.item(CommandItem::new().label(idea).icon(Lucide::Sparkles));
            }
            let mut go = CommandGroup::new().label("Go to");
            for (label, icon, _) in GO {
                go = go.item(CommandItem::new().label(label).icon(icon));
            }
            let show = CommandGroup::new().label("Show").item(CommandItem::new().label("Show what changed").icon(Lucide::ScanEye).checked(show_added));
            dialog.w(px(620.)).p_0().close_button(false).child(
                Command::new(&st)
                    .placeholder("Say what you want, or where to go")
                    .group(ask)
                    .group(go)
                    .group(show)
                    .on_confirm(move |ix, window, cx| {
                        window.close_dialog(cx);
                        let typed = typed.clone();
                        let _ = me.update(cx, |this, cx| this.palette_pick(ix.section, ix.row, &typed, window, cx));
                    }),
            )
        });
        state.update(cx, |s, cx| s.focus(window, cx));
    }

    fn palette_pick(&mut self, section: usize, row: usize, typed: &str, window: &mut Window, cx: &mut Context<Self>) {
        match section {
            0 => {
                let words = if !typed.is_empty() && row == 0 { Some(typed.to_string()) } else { IDEAS.get(row - usize::from(!typed.is_empty())).map(|s| s.to_string()) };
                if let Some(words) = words {
                    self.start_build(words.clone());
                    toast(window, cx, "Your agent is on it", format!("Building “{words}” on your plan. Follow it in Add anything."));
                }
            }
            1 => {
                if let Some((_, _, v)) = GO.get(row) {
                    self.view = *v;
                }
            }
            _ => self.show_added = !self.show_added,
        }
        cx.notify();
    }
}

fn about_body(a: &Added, pal: &Pal) -> Div {
    v_flex()
        .gap_3()
        .child(div().text_xs().font_semibold().text_color(c_added()).child(if a.suggested { "SUGGESTED BY DONERIGHT FROM HOW YOU WORK" } else if a.replaces.is_some() { "YOUR VERSION OF A SHIPPED VIEW, FROM YOUR WORDS" } else { "BUILT IN ONE SHOT FROM YOUR WORDS" }))
        .child(div().p_3().rounded(px(10.)).border_l_2().border_color(c_added()).bg(c_added().opacity(0.07)).text_sm().child(format!("“{}”", a.words)))
        .child(
            div().flex_shrink_0().child(
                DescriptionList::horizontal()
                    .columns(1)
                    .bordered(true)
                    .label_width(px(104.))
                    .item("Built by", SharedString::from(format!("{}, on your plan", a.agent)), 1)
                    .item("It cost", a.cost, 1)
                    .item("Checked", "conformance, its own tests, a render, a dry run on your data, access", 1)
                    .item("Added", a.when, 1)
                    .item("Shows up in", SharedString::from(a.places()), 1),
            ),
        )
        .child(v_flex().gap_1().child(section("IT CAN", pal)).children(a.can.iter().map(|t| div().text_xs().child(format!("✓ {t}")))))
        .child(v_flex().gap_1().child(section("IT CAN’T", pal)).children(a.cant.iter().map(|t| div().text_xs().text_color(pal.muted_fg).child(format!("✗ {t}")))))
        .child(v_flex().gap_1().child(section("HISTORY", pal)).children(a.history.iter().map(|(v, what)| {
            h_flex().gap_2().items_start().child(div().w(px(40.)).flex_shrink_0().text_xs().font_semibold().child(*v)).child(div().flex_1().text_xs().text_color(pal.muted_fg).child(*what))
        })))
        .child(div().text_xs().text_color(pal.muted_fg).child("Its panel is a script the app loads through gpui-shell, with no rebuild. The app draws it with the same components as everything else, so it looks native and sits where you put it, and it gets no access beyond the grant above. If it changes a shipped view, the shipped one stays one click away."))
}

fn bounces_panel(pal: &Pal) -> impl IntoElement {
    let rows = [
        ("Stripe", "Receipt for order #1042", "mailbox full", St::Back),
        ("Stripe", "Receipt for order #1039", "address not found", St::You),
        ("Postmark", "Weekly summary", "delivered after a retry", St::Ok),
    ];
    v_flex().gap_2().p_1().children(rows.into_iter().enumerate().map(|(i, (from, what, why, st))| {
        h_flex()
            .gap_3()
            .items_center()
            .py_2()
            .px_3()
            .rounded(px(10.))
            .border_1()
            .border_color(pal.border)
            .bg(pal.card)
            .child(Icon::from(Lucide::Mail).size_4().text_color(pal.muted_fg))
            .child(div().w(px(80.)).text_sm().font_semibold().child(from))
            .child(div().flex_1().text_sm().child(what))
            .child(match st { St::Ok => Tag::success(), St::You => Tag::danger(), _ => Tag::warning() }.small().child(why))
            .child(Button::new(("mail", i)).ghost().xsmall().icon(Lucide::ExternalLink).label("Open in Mail"))
    }))
}

fn buddy_panel(pal: &Pal) -> impl IntoElement {
    v_flex()
        .gap_2()
        .p_1()
        .child(div().text_sm().child("Hold ⌥Space and ask. It points at what you mean, right on your screen."))
        .children(["9:40 am · pointed at the coupon field in the checkout preview", "Yesterday · pointed at the failing row in the test report", "Monday · pointed at Settings › Privacy › Screen Recording"].into_iter().map(|t| {
            h_flex().gap_2().items_center().child(Icon::from(Lucide::MousePointer2).size_3p5().text_color(pal.muted_fg)).child(div().text_xs().text_color(pal.muted_fg).child(t))
        }))
}

fn other_panel(pal: &Pal) -> impl IntoElement {
    v_flex()
        .items_center()
        .gap_2()
        .py_8()
        .child(Icon::from(Lucide::Sparkles).size_6().text_color(c_added()))
        .child(div().text_sm().child("Your agent built this panel from your words."))
        .child(div().text_xs().text_color(pal.muted_fg).child("It fills in as soon as there’s something to show."))
}
