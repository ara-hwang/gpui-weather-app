#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! 릴리스 빌드에서는 콘솔 창 없이 GUI만 띄운다.
//! 디버그 빌드에서는 콘솔을 유지해 로그를 볼 수 있다.

mod app;
mod weather;

use app::WeatherApp;
use gpui_kit::component::Root;
use gpui_kit::*;

fn main() {
    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);

    app.run(move |cx: &mut App| {
        gpui_kit::init(cx);

        cx.spawn(async move |cx| {
            let bounds = cx.update(|cx| Bounds::centered(None, size(px(1024.), px(680.)), cx));
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("한국 날씨".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| WeatherApp::new());
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .expect("윈도우 열기 실패");
        })
        .detach();
    });
}
