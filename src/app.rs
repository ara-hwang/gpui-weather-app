//! 메인 GPUI 뷰: 사이드바(고정 도시) + 현재/시간별/주간 예보.

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::StyledExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::*;

use crate::weather::{CITIES, City, WeatherData, fetch_weather};

#[derive(Debug, Clone)]
enum LoadState {
    Loading,
    Loaded(WeatherData),
    Error(String),
}

pub struct WeatherApp {
    selected: usize,
    state: LoadState,
    started: bool,
}

impl WeatherApp {
    pub fn new() -> Self {
        Self {
            selected: 0,
            state: LoadState::Loading,
            started: false,
        }
    }

    fn selected_city(&self) -> City {
        CITIES[self.selected]
    }

    fn select_city(&mut self, ix: usize, cx: &mut Context<Self>) {
        if ix == self.selected {
            return;
        }
        self.selected = ix;
        self.refresh(cx);
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        self.state = LoadState::Loading;
        cx.notify();

        let city = self.selected_city();
        cx.spawn(async move |weak, cx| {
            let result = cx
                .background_spawn(async move { fetch_weather(&city) })
                .await;
            let _ = weak.update(cx, |view, cx| {
                match result {
                    Ok(data) => view.state = LoadState::Loaded(data),
                    Err(e) => view.state = LoadState::Error(format!("불러오기 실패: {e:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("sidebar")
            .test_support()
            .w(px(168.))
            .flex_shrink_0()
            .gap_1()
            .p_2()
            .child(
                div()
                    .px_2()
                    .py_1()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("도시"),
            )
            .children(CITIES.iter().enumerate().map(|(ix, city)| {
                let active = ix == self.selected;
                let btn = Button::new(format!("city-{ix}"))
                    .label(city.name)
                    .small();
                let btn = if active { btn.primary() } else { btn.ghost() };
                btn.on_click(cx.listener(move |this, _, _, cx| {
                    this.select_city(ix, cx);
                }))
            }))
    }

    fn render_current(&self, data: &WeatherData, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .p_4()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} · {}", self.selected_city().name, data.updated_at)),
                    )
                    .child(
                        Button::new("refresh")
                            .small()
                            .ghost()
                            .label("새로고침")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.refresh(cx);
                            })),
                    ),
            )
            .child(
                h_flex()
                    .items_end()
                    .gap_3()
                    .child(div().text_3xl().child(data.icon))
                    .child(
                        v_flex().gap_1().child(
                            div()
                                .text_2xl()
                                .font_bold()
                                .child(format!("{:.1}°C", data.temp)),
                        ).child(
                            div()
                                .text_lg()
                                .child(data.desc),
                        ),
                    ),
            )
            .child(
                h_flex()
                    .gap_4()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(div().child(format!("체감 {:.1}°C", data.apparent)))
                    .child(div().child(format!("습도 {}%", data.humidity)))
                    .child(div().child(format!("풍속 {:.1}m/s", data.wind))),
            )
    }

    fn render_hourly(&self, data: &WeatherData, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .p_3()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .child(div().text_sm().child("시간별 예보"))
            .child(
                h_flex()
                    .gap_2()
                    .children(data.hourly.iter().take(12).map(|h| {
                        v_flex()
                            .flex_1()
                            .items_center()
                            .gap_1()
                            .p_2()
                            .rounded(cx.theme().radius)
                            .text_sm()
                            .child(div().child(h.hour_label.clone()))
                            .child(div().text_xl().child(h.icon))
                            .child(div().font_bold().child(format!("{:.0}°", h.temp)))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{}%", h.precip_prob)),
                            )
                    })),
            )
    }

    fn render_daily(&self, data: &WeatherData, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("daily")
            .test_support()
            .gap_1()
            .p_3()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .child(div().text_sm().child("주간 예보"))
            .children(data.daily.iter().map(|d| {
                h_flex()
                    .items_center()
                    .justify_between()
                    .py_1()
                    .px_2()
                    .text_sm()
                    .child(
                        div()
                            .w(px(84.))
                            .child(format!("{} ({})", d.date_label, d.weekday_label)),
                    )
                    .child(div().w(px(28.)).child(d.icon))
                    .child(
                        div()
                            .flex_1()
                            .text_color(cx.theme().muted_foreground)
                            .child(d.desc),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{}%", d.precip_prob)),
                    )
                    .child(
                        div()
                            .w(px(110.))
                            .text_right()
                            .child(format!("{:.0}° / {:.0}°", d.max, d.min)),
                    )
            }))
    }
}

impl Render for WeatherApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 첫 프레임에서 데이터 로딩 시작 (entity가 존재하는 시점).
        if !self.started {
            self.started = true;
            self.refresh(cx);
        }

        let main = match &self.state {
            LoadState::Loading => v_flex()
                .id("main")
                .test_support()
                .flex_1()
                .min_w_0()
                .min_h_0()
                .items_center()
                .justify_center()
                .gap_2()
                .p_6()
                .child(div().text_lg().child("날씨 불러오는 중…"))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(self.selected_city().name),
                )
                .into_any_element(),
            LoadState::Error(msg) => v_flex()
                .id("main")
                .test_support()
                .flex_1()
                .min_w_0()
                .min_h_0()
                .items_center()
                .justify_center()
                .gap_3()
                .p_6()
                .child(div().text_lg().child("날씨를 가져오지 못했어요"))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(msg.clone()),
                )
                .child(
                    Button::new("retry")
                        .primary()
                        .label("다시 시도")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.refresh(cx);
                        })),
                )
                .into_any_element(),
            LoadState::Loaded(data) => v_flex()
                .id("main")
                .test_support()
                .flex_1()
                .min_w_0()
                .min_h_0()
                .child(
                    v_flex()
                        .size_full()
                        .overflow_y_scrollbar()
                        .gap_3()
                        .p_4()
                        .child(self.render_current(data, cx))
                        .child(self.render_hourly(data, cx))
                        .child(self.render_daily(data, cx)),
                )
                .into_any_element(),
        };

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .font_family("Segoe UI")
            .child(
                div()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(div().font_bold().child("한국 날씨")),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .items_stretch()
                    .child(self.render_sidebar(cx))
                    .child(main),
            )
    }
}

#[cfg(test)]
mod tests {
    // `super::*`를 쓰면 부모의 `use gpui_kit::*`가 `test` 속성 매크로까지
    // 가져와, 매크로가 생성한 `#[test]`가 자기 자신으로 해석되어 재귀한다.
    use super::{LoadState, WeatherApp, WeatherData};
    use crate::weather::{DailyPoint, HourlyPoint};
    use gpui_kit::component::Root;
    use gpui_kit::test::TestWindowExt as _;
    use gpui_kit::{
        AnyWindowHandle, AppContext as _, Bounds, Pixels, ScrollDelta, TestAppContext, Window,
        point, px, size,
    };

    fn sample_weather() -> WeatherData {
        WeatherData {
            temp: 21.5,
            apparent: 22.0,
            humidity: 60,
            wind: 3.2,
            code: 1,
            desc: "대체로 맑음",
            icon: "🌤️",
            hourly: (0..12)
                .map(|i| HourlyPoint {
                    hour_label: format!("{i}시"),
                    temp: 20.0 + i as f64,
                    precip_prob: i * 5,
                    icon: "☀️",
                    desc: "맑음",
                })
                .collect(),
            daily: (0..7)
                .map(|i| DailyPoint {
                    date_label: format!("9/{}", 11 + i),
                    weekday_label: "금".to_string(),
                    max: 25.0,
                    min: 18.0,
                    precip_prob: 10,
                    icon: "☀️",
                    desc: "맑음",
                })
                .collect(),
            updated_at: "12:00 갱신".to_string(),
        }
    }

    fn observe(window: &Window) -> (Bounds<Pixels>, Bounds<Pixels>) {
        (
            window.find("sidebar").bounds(),
            window.find("main").bounds(),
        )
    }

    /// 로딩 중과 로딩 완료 후에 사이드바/메인의 폭과 위치가 그대로여야 한다.
    /// 회귀 방지 1: 시간별 예보의 고정 폭 아이템이 메인의 min-content 폭을
    /// 키우면 사이드바(flex-shrink: 1)가 눌려 폭이 흔들린다.
    /// 회귀 방지 2: 콘텐츠 높이가 커질 때 행이 늘어나면 items_center 때문에
    /// 사이드바가 아래로 밀린다.
    #[gpui_kit::test]
    fn layout_stays_stable_and_main_scrolls(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);

        let app = std::rc::Rc::new(std::cell::RefCell::new(None));
        let handle = cx.open_window(size(px(1024.), px(680.)), {
            let app = app.clone();
            move |window, cx| {
                let view = cx.new(|_| {
                    let mut app = WeatherApp::new();
                    // 네트워크 요청 없이 상태를 직접 제어한다.
                    app.started = true;
                    app
                });
                *app.borrow_mut() = Some(view.clone());
                Root::new(view, window, cx)
            }
        });
        let handle: AnyWindowHandle = handle.into();
        let view = app.borrow().clone().expect("view를 캡처해야 합니다");

        let (loading_sidebar, loading_main) = cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                observe(window)
            })
            .unwrap();

        cx.update(|cx| {
            view.update(cx, |app, cx| {
                app.state = LoadState::Loaded(sample_weather());
                cx.notify();
            });
        });

        let (loaded_sidebar, loaded_main) = cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                observe(window)
            })
            .unwrap();

        assert_eq!(
            loading_sidebar.size.width, loaded_sidebar.size.width,
            "로딩/완료 사이에 사이드바 폭이 달라지면 안 됩니다"
        );
        assert_eq!(
            loading_main.size.width, loaded_main.size.width,
            "로딩/완료 사이에 메인 폭이 달라지면 안 됩니다"
        );
        assert_eq!(
            loading_sidebar.origin, loaded_sidebar.origin,
            "로딩/완료 사이에 사이드바 위치가 달라지면 안 됩니다"
        );
        assert_eq!(
            loading_main.origin.y, loaded_main.origin.y,
            "로딩/완료 사이에 메인 시작 위치가 달라지면 안 됩니다"
        );
        assert_eq!(loading_sidebar.size.width, px(168.));

        // 메인 내부 스크롤: 완료 후 하단 주간 예보까지 휠로 내려갈 수 있어야 한다.
        cx.update_window(handle, |_, window, cx| {
            let before = window.find("daily").bounds().top();
            window.scroll(
                "main",
                ScrollDelta::Pixels(point(px(0.), px(-160.))),
                cx,
            );
            let after = window.find("daily").bounds().top();
            assert!(
                after < before,
                "메인 내부 스크롤이 동작해야 합니다: {before:?} -> {after:?}"
            );
        })
        .unwrap();
    }
}
