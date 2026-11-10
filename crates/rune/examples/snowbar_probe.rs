use rune::{App, AppSpec, Color, Cx, El, Key, KeyEvent, KeyPolicy, OverlayInput, OverlayOptions, OverlayPlacement, OverlaySpec, Tx, WindowSpec, el, text};

static MENU: std::sync::OnceLock<std::sync::Mutex<Option<rune::OverlayHandle<u32>>>> = std::sync::OnceLock::new();

fn bar_view(clicks: &u32, _: &mut Cx<u32>) -> El {
    el().w_full().h_full().fill(Color { r: 0.08, g: 0.18, b: 0.28, a: 1.0 })
        .child(el().padding(8.0).hit().child(text(format!("Rune reservation test — click here: {clicks} | Menu")).color(Color::WHITE))
            .on_press(move |count: &mut u32| {
                *count += 1;
                if let Some(menu) = MENU.get().and_then(|m| m.lock().unwrap().clone()) { menu.show(); menu.request_keys(); }
            }))
}

fn menu_view(_: &u32, _: &mut Cx<u32>) -> El {
    el().w_full().h_full().fill(Color { r: 0.16, g: 0.12, b: 0.24, a: 0.98 })
        .child(el().padding(12.0).hit().child(text("Menu — click to dismiss").color(Color::WHITE))
            .on_press(move |_: &mut u32| {
                if let Some(menu) = MENU.get().and_then(|m| m.lock().unwrap().clone()) { menu.hide(); }
            }))
}

fn menu_key(_: &mut u32, event: &KeyEvent, _: &mut Tx) -> bool {
    if event.key == Key::Escape {
        if let Some(menu) = MENU.get().and_then(|m| m.lock().unwrap().clone()) { menu.hide(); }
        return true;
    }
    false
}

fn main() {
    rune::run_windows(|_| {
        let mut app = App::new(AppSpec::new("Snowbar probe", (400.0, 40.0), || 0_u32, bar_view)).windowless();
        let bar = app.overlay(OverlaySpec::new("snowbar-probe", (400.0, 40.0), bar_view).options(OverlayOptions {
            input: OverlayInput::All, ..Default::default()
        }).wayland_top_reserved(40), 0);
        let menu = app.overlay(OverlaySpec::new("snowbar-menu", (240.0, 120.0), menu_view).options(OverlayOptions {
            input: OverlayInput::All, keys: KeyPolicy::OnDemand,
            placement: Some(OverlayPlacement::Rect { x: 12.0, y: 0.0, width: 240.0, height: 120.0 }), ..Default::default()
        }).on_key(menu_key), 0);
        let _ = MENU.set(std::sync::Mutex::new(Some(menu.clone())));
        bar.show();
        let app_handle = app.handle();
        std::thread::spawn(move || { std::thread::sleep(std::time::Duration::from_secs(180)); app_handle.quit(); });
        vec![WindowSpec::from_app("probe", app)]
    });
}
