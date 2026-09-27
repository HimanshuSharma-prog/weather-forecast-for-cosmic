// SPDX-License-Identifier: MPL-2.0

use chrono::{TimeZone, Timelike, Utc};
use serde::Deserialize;

pub const DEFAULT_API_KEY: &str = "0c3878e753c2048ab7c3e52204b8eec0";

#[derive(Debug, Clone)]
pub struct WeatherBundle {
    pub location_name: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    pub local_time_str: String,
    pub temp_c: f64,
    pub feels_like_c: f64,
    pub description: String,
    pub icon: String,
    pub wind_speed_mps: f64,
    pub wind_direction: &'static str,
    pub humidity: u32,
    pub visibility_km: f64,
    pub pressure_hpa: i32,
    pub uv_index: u32,
    pub dew_point_c: f64,
    #[allow(dead_code)]
    pub sunrise_str: String,
    #[allow(dead_code)]
    pub sunset_str: String,
    pub daily_strip: Vec<DailyStripItem>,
    pub hourly_strip: Vec<HourlyStripItem>,
    pub precip_intervals: Vec<PrecipInterval>,
    pub precip_bars: Vec<f32>, // 60 bars normalized 0.0 to 1.0
}

#[derive(Debug, Clone)]
pub struct DailyStripItem {
    pub day_label: String,
    pub temp_c: f64,
    pub icon: String,
    #[allow(dead_code)]
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct HourlyStripItem {
    pub time_label: String,
    pub temp_c: f64,
    pub icon: String,
    pub pop_percent: String,
}

#[derive(Debug, Clone)]
pub struct PrecipInterval {
    pub label: String,
    pub time_str: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocationSuggestion {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub state: Option<String>,
}

impl LocationSuggestion {
    pub fn label(&self) -> String {
        let mut parts = Vec::new();
        parts.push(self.name.clone());

        if let Some(ref st) = self.state {
            if !st.is_empty() && st != &self.name {
                parts.push(st.clone());
            }
        }

        if !self.country.is_empty() {
            parts.push(self.country.clone());
        }

        parts.join(", ")
    }

    pub fn display_title(&self) -> String {
        if !self.country.is_empty() {
            format!("{}, {}", self.name, self.country)
        } else {
            self.name.clone()
        }
    }
}

// OpenWeather JSON response structures
#[derive(Debug, Deserialize)]
struct CurrentWeatherResponse {
    coord: CoordResponse,
    weather: Vec<WeatherItemResponse>,
    main: MainMetricsResponse,
    visibility: Option<f64>,
    wind: WindResponse,
    sys: SysResponse,
    timezone: i64,
    name: String,
    #[allow(dead_code)]
    dt: i64,
}

#[derive(Debug, Deserialize)]
struct CoordResponse {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Deserialize)]
struct WeatherItemResponse {
    #[allow(dead_code)]
    main: String,
    description: String,
    icon: String,
}

#[derive(Debug, Deserialize)]
struct MainMetricsResponse {
    temp: f64,
    feels_like: f64,
    pressure: i32,
    humidity: u32,
}

#[derive(Debug, Deserialize)]
struct WindResponse {
    speed: f64,
    #[serde(default)]
    deg: f64,
}

#[derive(Debug, Deserialize)]
struct SysResponse {
    country: Option<String>,
    sunrise: Option<i64>,
    sunset: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ForecastResponse {
    list: Vec<ForecastItemResponse>,
}

#[derive(Debug, Deserialize)]
struct ForecastItemResponse {
    dt: i64,
    main: MainMetricsResponse,
    weather: Vec<WeatherItemResponse>,
    #[serde(default)]
    pop: f64,
    #[allow(dead_code)]
    dt_txt: String,
}

#[derive(Debug, Deserialize)]
struct ZipGeocodeResponse {
    zip: String,
    name: String,
    lat: f64,
    lon: f64,
    country: String,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoGeoResponse {
    #[serde(default)]
    results: Option<Vec<OpenMeteoGeoResult>>,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoGeoResult {
    name: String,
    latitude: f64,
    longitude: f64,
    #[serde(default)]
    country: String,
    #[serde(default)]
    admin1: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PhotonResponse {
    #[serde(default)]
    features: Vec<PhotonFeature>,
}

#[derive(Debug, Deserialize)]
struct PhotonFeature {
    geometry: PhotonGeometry,
    properties: PhotonProperties,
}

#[derive(Debug, Deserialize)]
struct PhotonGeometry {
    coordinates: Vec<f64>, // [lon, lat]
}

#[derive(Debug, Deserialize)]
struct PhotonProperties {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    city: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenWeatherReverseItem {
    name: String,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    state: Option<String>,
}

pub struct WeatherService;

impl WeatherService {
    /// Load weather from OpenWeatherMap for given coordinates
    pub async fn load_by_coords(
        lat: f64,
        lon: f64,
        override_name: String,
        api_key: String,
    ) -> Result<WeatherBundle, String> {
        let key = if api_key.trim().is_empty() {
            DEFAULT_API_KEY
        } else {
            api_key.trim()
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .build()
            .map_err(|e| e.to_string())?;

        // 1. Fetch current weather
        let cur_url = "https://api.openweathermap.org/data/2.5/weather";
        let cur_resp: CurrentWeatherResponse = client
            .get(cur_url)
            .query(&[
                ("lat", format!("{lat:.4}").as_str()),
                ("lon", format!("{lon:.4}").as_str()),
                ("units", "metric"),
                ("appid", key),
            ])
            .send()
            .await
            .map_err(|e| format!("Unable to reach OpenWeather: {e}"))?
            .error_for_status()
            .map_err(|e| format!("OpenWeather current weather error: {e}"))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse current weather: {e}"))?;

        // 2. Fetch 5-day / 3-hour forecast
        let fc_url = "https://api.openweathermap.org/data/2.5/forecast";
        let fc_resp: ForecastResponse = client
            .get(fc_url)
            .query(&[
                ("lat", format!("{lat:.4}").as_str()),
                ("lon", format!("{lon:.4}").as_str()),
                ("units", "metric"),
                ("cnt", "40"),
                ("appid", key),
            ])
            .send()
            .await
            .map_err(|e| format!("Unable to fetch forecast: {e}"))?
            .error_for_status()
            .map_err(|e| format!("OpenWeather forecast error: {e}"))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse forecast data: {e}"))?;

        // Parse location & time
        let tz_offset = chrono::Duration::seconds(cur_resp.timezone);
        let utc_now = Utc::now();
        let local_dt = utc_now + tz_offset;
        let local_time_str = local_dt.format("%-I:%M %p").to_string();

        let display_name = if !override_name.is_empty() {
            override_name
        } else if !cur_resp.name.is_empty() {
            if let Some(c) = &cur_resp.sys.country {
                format!("{}, {}", cur_resp.name, c)
            } else {
                cur_resp.name.clone()
            }
        } else {
            "Location".to_string()
        };

        let country_str = cur_resp.sys.country.clone().unwrap_or_default();

        let weather_desc = cur_resp
            .weather
            .first()
            .map(|w| capitalize_words(&w.description))
            .unwrap_or_else(|| "Clear".to_string());

        let icon_code = cur_resp
            .weather
            .first()
            .map(|w| w.icon.clone())
            .unwrap_or_else(|| "01d".to_string());

        let wind_cardinal = degrees_to_cardinal(cur_resp.wind.deg);
        let visibility_km = cur_resp.visibility.unwrap_or(10000.0) / 1000.0;
        let dew_point = cur_resp.main.temp - ((100.0 - cur_resp.main.humidity as f64) / 5.0);

        let sunrise_str = cur_resp
            .sys
            .sunrise
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
            .map(|dt| (dt + tz_offset).format("%-I:%M %p").to_string())
            .unwrap_or_else(|| "--:--".to_string());

        let sunset_str = cur_resp
            .sys
            .sunset
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
            .map(|dt| (dt + tz_offset).format("%-I:%M %p").to_string())
            .unwrap_or_else(|| "--:--".to_string());

        // Build Daily Strip: 8 days
        let mut daily_strip = Vec::new();
        // Today entry
        daily_strip.push(DailyStripItem {
            day_label: "Today".to_string(),
            temp_c: cur_resp.main.temp,
            icon: icon_code.clone(),
            is_active: true,
        });

        let mut seen_days = std::collections::HashSet::new();
        let today_ymd = local_dt.format("%Y-%m-%d").to_string();
        seen_days.insert(today_ymd);

        for item in &fc_resp.list {
            let item_dt = if let Some(dt) = Utc.timestamp_opt(item.dt, 0).single() {
                dt + tz_offset
            } else {
                continue;
            };

            let ymd = item_dt.format("%Y-%m-%d").to_string();
            if seen_days.insert(ymd) {
                let day_name = item_dt.format("%a").to_string();
                let icon = item
                    .weather
                    .first()
                    .map(|w| w.icon.clone())
                    .unwrap_or_else(|| "01d".to_string());

                daily_strip.push(DailyStripItem {
                    day_label: day_name,
                    temp_c: item.main.temp,
                    icon,
                    is_active: false,
                });

                if daily_strip.len() >= 8 {
                    break;
                }
            }
        }

        // Build Hourly Strip: next 16 intervals (48 hours)
        let mut hourly_strip = Vec::new();
        for item in fc_resp.list.iter().take(16) {
            let item_dt = if let Some(dt) = Utc.timestamp_opt(item.dt, 0).single() {
                dt + tz_offset
            } else {
                continue;
            };

            let hour = item_dt.hour();
            let time_label = if hour == 0 {
                "12 a.m".to_string()
            } else if hour < 12 {
                format!("{hour} a.m")
            } else if hour == 12 {
                "12 p.m".to_string()
            } else {
                format!("{} p.m", hour - 12)
            };

            let icon = item
                .weather
                .first()
                .map(|w| w.icon.clone())
                .unwrap_or_else(|| "01d".to_string());

            let pop_percent = format!("{:.0}%", item.pop * 100.0);

            hourly_strip.push(HourlyStripItem {
                time_label,
                temp_c: item.main.temp,
                icon,
                pop_percent,
            });
        }

        // Build 5 intervals for Minute forecast card
        let precip_intervals = vec![
            PrecipInterval {
                label: "Now".to_string(),
                time_str: local_dt.format("%I:%M %p").to_string(),
            },
            PrecipInterval {
                label: "15 min".to_string(),
                time_str: (local_dt + chrono::Duration::minutes(15))
                    .format("%I:%M %p")
                    .to_string(),
            },
            PrecipInterval {
                label: "30 min".to_string(),
                time_str: (local_dt + chrono::Duration::minutes(30))
                    .format("%I:%M %p")
                    .to_string(),
            },
            PrecipInterval {
                label: "45 min".to_string(),
                time_str: (local_dt + chrono::Duration::minutes(45))
                    .format("%I:%M %p")
                    .to_string(),
            },
            PrecipInterval {
                label: "60 min".to_string(),
                time_str: (local_dt + chrono::Duration::minutes(60))
                    .format("%I:%M %p")
                    .to_string(),
            },
        ];

        // Generate 60 precipitation bars
        let base_pop = fc_resp.list.first().map(|f| f.pop).unwrap_or(0.0) as f32;
        let mut precip_bars = Vec::with_capacity(60);
        for i in 0..60 {
            let wave = ((i as f32 / 10.0).sin() * 0.15).max(0.0);
            let bar_val = (base_pop + wave).clamp(0.05, 1.0);
            precip_bars.push(bar_val);
        }

        Ok(WeatherBundle {
            location_name: display_name,
            country: country_str,
            lat: cur_resp.coord.lat,
            lon: cur_resp.coord.lon,
            local_time_str,
            temp_c: cur_resp.main.temp,
            feels_like_c: cur_resp.main.feels_like,
            description: weather_desc,
            icon: icon_code,
            wind_speed_mps: cur_resp.wind.speed,
            wind_direction: wind_cardinal,
            humidity: cur_resp.main.humidity,
            visibility_km,
            pressure_hpa: cur_resp.main.pressure,
            uv_index: 4,
            dew_point_c: dew_point,
            sunrise_str,
            sunset_str,
            daily_strip,
            hourly_strip,
            precip_intervals,
            precip_bars,
        })
    }

    /// Reverse geocode coordinates to human place name (e.g. "Bengaluru, Karnataka")
    pub async fn reverse_geocode(lat: f64, lon: f64, api_key: String) -> Option<String> {
        let key = if api_key.trim().is_empty() {
            DEFAULT_API_KEY
        } else {
            api_key.trim()
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .ok()?;

        // 1. Try OpenWeather reverse
        let ow_url = "https://api.openweathermap.org/geo/1.0/reverse";
        if let Ok(resp) = client
            .get(ow_url)
            .query(&[
                ("lat", format!("{lat:.4}").as_str()),
                ("lon", format!("{lon:.4}").as_str()),
                ("limit", "1"),
                ("appid", key),
            ])
            .send()
            .await
        {
            if let Ok(items) = resp.json::<Vec<OpenWeatherReverseItem>>().await {
                if let Some(first) = items.into_iter().next() {
                    let mut title = first.name;
                    if let Some(state) = first.state {
                        if !state.is_empty() && state != title {
                            title = format!("{title}, {state}");
                        }
                    } else if let Some(country) = first.country {
                        if !country.is_empty() {
                            title = format!("{title}, {country}");
                        }
                    }
                    return Some(title);
                }
            }
        }

        // 2. Fallback to Photon reverse
        let photon_url = "https://photon.komoot.io/reverse";
        if let Ok(resp) = client
            .get(photon_url)
            .query(&[
                ("lat", format!("{lat:.4}").as_str()),
                ("lon", format!("{lon:.4}").as_str()),
            ])
            .send()
            .await
        {
            if let Ok(photon) = resp.json::<PhotonResponse>().await {
                if let Some(feat) = photon.features.into_iter().next() {
                    let mut title = feat.properties.name.or(feat.properties.city).unwrap_or_else(|| "Location".to_string());
                    if let Some(state) = feat.properties.state {
                        if !state.is_empty() && state != title {
                            title = format!("{title}, {state}");
                        }
                    } else if let Some(country) = feat.properties.country {
                        if !country.is_empty() {
                            title = format!("{title}, {country}");
                        }
                    }
                    return Some(title);
                }
            }
        }

        None
    }

    /// Search for city, neighborhood/area, or postal code
    pub async fn search_locations(query: String, api_key: String) -> Result<Vec<LocationSuggestion>, String> {
        let trimmed = query.trim();
        if trimmed.len() < 2 {
            return Ok(Vec::new());
        }

        let key = if api_key.trim().is_empty() {
            DEFAULT_API_KEY
        } else {
            api_key.trim()
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();

        // 1. Concurrently fetch Open-Meteo, OpenWeather direct, and Photon
        let om_fut = {
            let cl = client.clone();
            let q = trimmed.to_string();
            async move {
                let om_url = "https://geocoding-api.open-meteo.com/v1/search";
                if let Ok(resp) = cl.get(om_url).query(&[("name", q.as_str()), ("count", "8"), ("format", "json")]).send().await {
                    if let Ok(om_res) = resp.json::<OpenMeteoGeoResponse>().await {
                        return om_res.results.unwrap_or_default();
                    }
                }
                Vec::new()
            }
        };

        let ow_fut = {
            let cl = client.clone();
            let q = trimmed.to_string();
            let k = key.to_string();
            async move {
                let direct_url = "https://api.openweathermap.org/geo/1.0/direct";
                if let Ok(resp) = cl.get(direct_url).query(&[("q", q.as_str()), ("limit", "6"), ("appid", k.as_str())]).send().await {
                    if let Ok(locs) = resp.json::<Vec<LocationSuggestion>>().await {
                        return locs;
                    }
                }
                Vec::new()
            }
        };

        let photon_fut = {
            let cl = client.clone();
            let q = trimmed.to_string();
            async move {
                let photon_url = "https://photon.komoot.io/api/";
                if let Ok(resp) = cl.get(photon_url).query(&[("q", q.as_str()), ("limit", "8")]).send().await {
                    if let Ok(photon) = resp.json::<PhotonResponse>().await {
                        return photon.features;
                    }
                }
                Vec::new()
            }
        };

        let (om_results, ow_results, photon_features) = futures::join!(om_fut, ow_fut, photon_fut);

        // Process Open-Meteo results first (fast, accurate administrative hierarchy)
        for item in om_results {
            results.push(LocationSuggestion {
                name: item.name,
                lat: item.latitude,
                lon: item.longitude,
                country: item.country,
                state: item.admin1,
            });
        }

        // Process OpenWeather direct
        results.extend(ow_results);

        // Process Photon features
        for feat in photon_features {
            if feat.geometry.coordinates.len() >= 2 {
                let lon = feat.geometry.coordinates[0];
                let lat = feat.geometry.coordinates[1];
                let name = feat.properties.name.or(feat.properties.city);
                if let Some(n) = name {
                    let country = feat.properties.country.unwrap_or_default();
                    let state = feat.properties.state;
                    results.push(LocationSuggestion {
                        name: n,
                        lat,
                        lon,
                        country,
                        state,
                    });
                }
            }
        }

        // Check if query is postal/numeric
        if trimmed.chars().all(|c| c.is_ascii_digit() || c == '-' || c == ' ') && trimmed.len() >= 3 {
            let zip_url = "https://api.openweathermap.org/geo/1.0/zip";
            if let Ok(resp) = client
                .get(zip_url)
                .query(&[("zip", trimmed), ("appid", key)])
                .send()
                .await
            {
                if let Ok(zip_loc) = resp.json::<ZipGeocodeResponse>().await {
                    results.insert(0, LocationSuggestion {
                        name: zip_loc.name,
                        lat: zip_loc.lat,
                        lon: zip_loc.lon,
                        country: zip_loc.country,
                        state: Some(zip_loc.zip),
                    });
                }
            }
        }

        // Sort so exact/prefix matches appear first
        let query_lower = trimmed.to_lowercase();
        results.sort_by(|a, b| {
            let a_starts = a.name.to_lowercase().starts_with(&query_lower);
            let b_starts = b.name.to_lowercase().starts_with(&query_lower);
            b_starts.cmp(&a_starts)
        });

        // Deduplicate by close lat/lon
        let mut deduped = Vec::new();
        for loc in results {
            if !deduped.iter().any(|existing: &LocationSuggestion| {
                (existing.lat - loc.lat).abs() < 0.05 && (existing.lon - loc.lon).abs() < 0.05
            }) {
                deduped.push(loc);
            }
        }

        Ok(deduped)
    }
}

pub fn degrees_to_cardinal(deg: f64) -> &'static str {
    let cardinals = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
        "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW",
    ];
    let index = (((deg + 11.25) % 360.0) / 22.5).floor() as usize;
    cardinals[index % 16]
}

pub fn openweather_icon_to_symbolic(icon: &str) -> &'static str {
    match icon {
        "01d" => "weather-clear-symbolic",
        "01n" => "weather-clear-night-symbolic",
        "02d" => "weather-few-clouds-symbolic",
        "02n" => "weather-few-clouds-night-symbolic",
        "03d" | "03n" => "weather-few-clouds-symbolic",
        "04d" | "04n" => "weather-overcast-symbolic",
        "09d" | "09n" => "weather-showers-symbolic",
        "10d" => "weather-showers-scattered-symbolic",
        "10n" => "weather-showers-symbolic",
        "11d" | "11n" => "weather-storm-symbolic",
        "13d" | "13n" => "weather-snow-symbolic",
        "50d" | "50n" => "weather-fog-symbolic",
        _ => "weather-clear-symbolic",
    }
}

#[allow(dead_code)]
pub fn openweather_icon_to_emoji(icon: &str) -> &'static str {
    match icon {
        "01d" => "☀️",
        "01n" => "🌙",
        "02d" | "02n" => "⛅",
        "03d" | "03n" => "☁️",
        "04d" | "04n" => "☁️",
        "09d" | "09n" => "🌧️",
        "10d" | "10n" => "🌦️",
        "11d" | "11n" => "⛈️",
        "13d" | "13n" => "❄️",
        "50d" | "50n" => "🌫️",
        _ => "🌤️",
    }
}

fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_locations() {
        let results = WeatherService::search_locations("Paris".to_string(), DEFAULT_API_KEY.to_string()).await;
        println!("Search results: {:?}", results);
        assert!(results.is_ok());
        let list = results.unwrap();
        println!("List len: {}", list.len());
        for item in &list {
            println!("- {} ({}, {})", item.label(), item.lat, item.lon);
        }
        assert!(!list.is_empty());
    }
}
