//! Headless snapshots (`--features snap`): opens the app in GPUI's test platform with
//! the real text system and the Metal renderer, plays a short script of real clicks,
//! keys and typing, and saves the frame as a PNG. Nothing is shown on screen.
//!
//!   DR_SNAP=out.png DR_START=flow DR_DO="click:show-added; wait:400; click:p-add" doneright-app
//!
//! Script steps, separated by `;`: `click:ID`, `hover:ID`, `press:KEYS`, `input:TEXT`,
//! `wait:MS`. An ID of `name#3` is the element id `("name", 3)`.

use super::*;
use gpui_kit::{HeadlessAppContext, test::TestWindowExt as _};
use std::sync::Arc;

fn id(s: &str) -> ElementId {
    match s.rsplit_once('#') {
        Some((name, n)) if n.parse::<u64>().is_ok() => ElementId::NamedInteger(SharedString::from(name.to_string()), n.parse().unwrap()),
        _ => ElementId::Name(SharedString::from(s.to_string())),
    }
}

pub(crate) fn run() {
    let out = std::env::var("DR_SNAP").unwrap();
    let script = std::env::var("DR_DO").unwrap_or_default();
    let mut cx = HeadlessAppContext::with_platform(
        gpui_kit::platform::current_platform(true).text_system(),
        Arc::new(gpui_kit::assets::AllAssets),
        gpui_kit::platform::current_headless_renderer,
    );
    cx.update(|cx| {
        gpui_kit::init(cx);
        cx.bind_keys([KeyBinding::new("cmd-k", OpenPalette, None)]);
    });
    let (handle, _) = cx
        .update(|cx| {
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds { origin: Default::default(), size: size(px(1440.), px(860.)) })),
                    focus: true,
                    show: false,
                    ..TitleBar::window_options()
                },
                cx,
                |window, cx| cx.new(|cx| DoneRight::new(window, cx)),
            )
        })
        .expect("open the headless window");
    let settle = |cx: &mut HeadlessAppContext, ms: u64| {
        let end = Instant::now() + Duration::from_millis(ms);
        loop {
            cx.advance_clock(Duration::from_millis(33));
            cx.run_until_parked();
            let _ = cx.update_window(handle, |_, window, cx| window.render_frame(cx));
            if Instant::now() >= end {
                break;
            }
            std::thread::sleep(Duration::from_millis(15));
        }
    };
    settle(&mut cx, 500);
    for step in script.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let (kind, arg) = step.split_once(':').unwrap_or((step, ""));
        let arg = arg.trim().to_string();
        if kind == "wait" {
            settle(&mut cx, arg.parse().unwrap_or(300));
            continue;
        }
        let res = cx.update_window(handle, |_, window, cx| match kind {
            "click" => window.click(id(&arg), cx),
            "hover" => window.hover(id(&arg), cx),
            "press" => window.press(&arg, cx),
            "input" => window.input(&arg, cx),
            other => eprintln!("snap: unknown step {other}"),
        });
        if let Err(e) = res {
            eprintln!("snap: {step}: {e}");
        }
        settle(&mut cx, 350);
    }
    // let open animations (toasts, sheets, dialogs, charts) finish in real time
    settle(&mut cx, 900);
    let img = cx.capture_screenshot(handle).expect("Metal rendering must be available");
    img.save(&out).expect("save the PNG");
    println!("snap: {out} {}x{}", img.width(), img.height());
}
