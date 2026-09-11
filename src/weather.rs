//! Open-Meteo 기반 날씨 데이터 모듈 (API 키 불필요, Asia/Seoul 고정).

use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};
use serde::Deserialize;

#[derive(Debug, Clone, Copy)]
pub struct City {
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
}

/// 고정 도시 목록 (특·광역시 + 제주).
pub const CITIES: &[City] = &[
    City { name: "서울", lat: 37.57, lon: 126.98 },
    City { name: "인천", lat: 37.46, lon: 126.70 },
    City { name: "대전", lat: 36.35, lon: 127.38 },
    City { name: "대구", lat: 35.87, lon: 128.60 },
    City { name: "광주", lat: 35.16, lon: 126.85 },
    City { name: "부산", lat: 35.18, lon: 129.08 },
    City { name: "울산", lat: 35.54, lon: 129.31 },
    City { name: "세종", lat: 36.48, lon: 127.29 },
    City { name: "제주", lat: 33.50, lon: 126.53 },
];

/// WMO weather_code -> (한글 설명, 이모지).
pub fn describe_code(code: i32) -> (&'static str, &'static str) {
    match code {
        0 => ("맑음", "☀️"),
        1 => ("대체로 맑음", "🌤️"),
        2 => ("부분적으로 흐림", "⛅"),
        3 => ("흐림", "☁️"),
        45 | 48 => ("안개", "🌫️"),
        51 => ("가벼운 이슬비", "🌦️"),
        53 => ("이슬비", "🌦️"),
        55 => ("짙은 이슬비", "🌧️"),
        56 | 57 => ("어는 이슬비", "🌧️"),
        61 => ("약한 비", "🌧️"),
        63 => ("비", "🌧️"),
        65 => ("강한 비", "⛈️"),
        66 | 67 => ("어는 비", "🌧️"),
        71 => ("약한 눈", "🌨️"),
        73 => ("눈", "❄️"),
        75 => ("강한 눈", "❄️"),
        77 => ("싸락눈", "🌨️"),
        80 => ("약한 소나기", "🌦️"),
        81 => ("소나기", "🌧️"),
        82 => ("강한 소나기", "⛈️"),
        85 | 86 => ("눈 소나기", "🌨️"),
        95 => ("천둥번개", "⛈️"),
        96 | 99 => ("우박 동반 천둥", "⛈️"),
        _ => ("알 수 없음", "🌡️"),
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HourlyPoint {
    pub hour_label: String,
    pub temp: f64,
    pub precip_prob: i64,
    pub icon: &'static str,
    pub desc: &'static str,
}

#[derive(Debug, Clone)]
pub struct DailyPoint {
    pub date_label: String,
    pub weekday_label: String,
    pub max: f64,
    pub min: f64,
    pub precip_prob: i64,
    pub icon: &'static str,
    pub desc: &'static str,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WeatherData {
    pub temp: f64,
    pub apparent: f64,
    pub humidity: i64,
    pub wind: f64,
    pub code: i32,
    pub desc: &'static str,
    pub icon: &'static str,
    pub hourly: Vec<HourlyPoint>,
    pub daily: Vec<DailyPoint>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    current: ApiCurrent,
    hourly: ApiHourly,
    daily: ApiDaily,
}

#[derive(Debug, Deserialize)]
struct ApiCurrent {
    temperature_2m: f64,
    relative_humidity_2m: i64,
    apparent_temperature: f64,
    weather_code: i32,
    wind_speed_10m: f64,
}

#[derive(Debug, Deserialize)]
struct ApiHourly {
    time: Vec<String>,
    temperature_2m: Vec<f64>,
    precipitation_probability: Vec<Option<i64>>,
    weather_code: Vec<i32>,
}

#[derive(Debug, Deserialize)]
struct ApiDaily {
    time: Vec<String>,
    weather_code: Vec<i32>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    precipitation_probability_max: Vec<Option<i64>>,
}

const WEEKDAY_KO: [&str; 7] = ["월", "화", "수", "목", "금", "토", "일"];

fn weekday_ko(date: &NaiveDate) -> &'static str {
    WEEKDAY_KO[date.weekday().num_days_from_monday() as usize]
}

/// "2026-09-11T14:00" -> "14시", 파싱 실패 시 원본 반환.
fn hour_label(iso: &str) -> String {
    iso.get(11..13)
        .and_then(|h| h.parse::<u32>().ok())
        .map(|h| format!("{h}시"))
        .unwrap_or_else(|| iso.to_string())
}

/// "2026-09-11" -> ("9/11", "목").
fn date_labels(iso: &str) -> (String, String) {
    match NaiveDate::parse_from_str(iso, "%Y-%m-%d") {
        Ok(d) => (
            format!("{}/{}", d.month(), d.day()),
            weekday_ko(&d).to_string(),
        ),
        Err(_) => (iso.to_string(), String::new()),
    }
}

/// Open-Meteo에서 해당 도시의 현재+시간별+주간 예보를 가져온다.
///
/// 백그라운드 스레드에서 호출되는 것을 전제로 한 블로킹 함수다.
pub fn fetch_weather(city: &City) -> Result<WeatherData> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m&hourly=temperature_2m,precipitation_probability,weather_code&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max&timezone=Asia%2FSeoul&wind_speed_unit=ms&forecast_days=7",
        city.lat, city.lon
    );

    let body = ureq::get(&url)
        .call()
        .map_err(|e| anyhow::anyhow!("날씨 요청 실패: {e}"))?
        .body_mut()
        .read_json::<ApiResponse>()
        .context("날씨 응답 파싱 실패")?;

    let (desc, icon) = describe_code(body.current.weather_code);

    let n_hourly = body
        .hourly
        .time
        .len()
        .min(24)
        .min(body.hourly.temperature_2m.len())
        .min(body.hourly.weather_code.len());
    let mut hourly = Vec::with_capacity(n_hourly);
    for i in 0..n_hourly {
        let (d, ic) = describe_code(body.hourly.weather_code[i]);
        hourly.push(HourlyPoint {
            hour_label: hour_label(&body.hourly.time[i]),
            temp: body.hourly.temperature_2m[i],
            precip_prob: body
                .hourly
                .precipitation_probability
                .get(i)
                .copied()
                .flatten()
                .unwrap_or(0),
            icon: ic,
            desc: d,
        });
    }
    let n_daily = body
        .daily
        .time
        .len()
        .min(7)
        .min(body.daily.temperature_2m_max.len())
        .min(body.daily.temperature_2m_min.len())
        .min(body.daily.weather_code.len());
    let mut daily = Vec::with_capacity(n_daily);
    for i in 0..n_daily {
        let (d, ic) = describe_code(body.daily.weather_code[i]);
        let (date_label, weekday_label) = date_labels(&body.daily.time[i]);
        daily.push(DailyPoint {
            date_label,
            weekday_label,
            max: body.daily.temperature_2m_max[i],
            min: body.daily.temperature_2m_min[i],
            precip_prob: body
                .daily
                .precipitation_probability_max
                .get(i)
                .copied()
                .flatten()
                .unwrap_or(0),
            icon: ic,
            desc: d,
        });
    }

    Ok(WeatherData {
        temp: body.current.temperature_2m,
        apparent: body.current.apparent_temperature,
        humidity: body.current.relative_humidity_2m,
        wind: body.current.wind_speed_10m,
        code: body.current.weather_code,
        desc,
        icon,
        hourly,
        daily,
        updated_at: chrono::Local::now().format("%H:%M 갱신").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_mapping_covers_main_cases() {
        assert_eq!(describe_code(0).0, "맑음");
        assert_eq!(describe_code(3).0, "흐림");
        assert_eq!(describe_code(63).0, "비");
        assert_eq!(describe_code(73).0, "눈");
        assert_eq!(describe_code(95).0, "천둥번개");
    }

    #[test]
    fn hour_label_parses() {
        assert_eq!(hour_label("2026-09-11T04:00"), "4시");
        assert_eq!(hour_label("2026-09-11T14:00"), "14시");
    }

    #[test]
    fn date_labels_weekday() {
        let (date, weekday) = date_labels("2026-09-11");
        assert_eq!(date, "9/11");
        assert_eq!(weekday, "금");
    }

    #[test]
    fn response_deserializes() {
        let json = serde_json::json!({
            "current": {"temperature_2m": 21.5, "relative_humidity_2m": 60, "apparent_temperature": 22.0, "weather_code": 1, "wind_speed_10m": 3.2},
            "hourly": {"time": ["2026-09-11T00:00"], "temperature_2m": [20.0], "precipitation_probability": [10], "weather_code": [0]},
            "daily": {"time": ["2026-09-11"], "weather_code": [0], "temperature_2m_max": [25.0], "temperature_2m_min": [18.0], "precipitation_probability_max": [5]}
        });
        let resp: ApiResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.current.temperature_2m, 21.5);
        assert_eq!(resp.hourly.time.len(), 1);
        assert_eq!(resp.daily.time.len(), 1);
    }
}
