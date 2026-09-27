// SPDX-License-Identifier: MPL-2.0
#![allow(dead_code)]

use cosmic::widget::icon::{self, Handle, Icon};

/// System-themed symbolic icons that automatically match the active COSMIC DE theme colors,
/// including high contrast, light, dark, and custom desktop palettes.

pub fn handle_wind(size: u16) -> Handle {
    icon::from_name("weather-windy-symbolic").size(size).into()
}

pub fn handle_humidity(size: u16) -> Handle {
    icon::from_name("weather-showers-symbolic").size(size).into()
}

pub fn handle_visibility(size: u16) -> Handle {
    icon::from_name("image-red-eye-symbolic").size(size).into()
}

const PRESSURE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M5.5 17.5 A 8.5 8.5 0 1 1 18.5 17.5" />
  <line x1="12" y1="5.5" x2="12" y2="7.5" />
  <line x1="6.5" y1="11" x2="8.5" y2="11.5" />
  <line x1="17.5" y1="11" x2="15.5" y2="11.5" />
  <line x1="12" y1="12" x2="16" y2="8" />
  <circle cx="12" cy="12" r="2" fill="currentColor" />
</svg>"#;

pub fn handle_pressure(_size: u16) -> Handle {
    let mut h = icon::from_svg_bytes(PRESSURE_SVG.as_bytes());
    h.symbolic = true;
    h
}

pub fn handle_uv_index(size: u16) -> Handle {
    icon::from_name("weather-clear-symbolic").size(size).into()
}

pub fn handle_dew_point(size: u16) -> Handle {
    icon::from_name("weather-few-clouds-symbolic").size(size).into()
}

pub fn handle_search(size: u16) -> Handle {
    icon::from_name("edit-find-symbolic").size(size).into()
}

pub fn handle_location(size: u16) -> Handle {
    icon::from_name("find-location-symbolic").size(size).into()
}

pub fn handle_star_filled(size: u16) -> Handle {
    icon::from_name("starred-symbolic").size(size).into()
}

pub fn handle_star_outline(size: u16) -> Handle {
    icon::from_name("non-starred-symbolic").size(size).into()
}

pub fn handle_refresh(size: u16) -> Handle {
    icon::from_name("view-refresh-symbolic").size(size).into()
}

pub fn icon_wind(size: u16) -> Icon {
    icon::from_name("weather-windy-symbolic").size(size).into()
}

pub fn icon_humidity(size: u16) -> Icon {
    icon::from_name("weather-showers-symbolic").size(size).into()
}

pub fn icon_visibility(size: u16) -> Icon {
    icon::from_name("image-red-eye-symbolic").size(size).into()
}

pub fn icon_pressure(size: u16) -> Icon {
    handle_pressure(size).icon().size(size)
}

pub fn icon_uv_index(size: u16) -> Icon {
    icon::from_name("weather-clear-symbolic").size(size).into()
}

pub fn icon_dew_point(size: u16) -> Icon {
    icon::from_name("weather-few-clouds-symbolic").size(size).into()
}

pub fn icon_search(size: u16) -> Icon {
    icon::from_name("edit-find-symbolic").size(size).into()
}

pub fn icon_location(size: u16) -> Icon {
    icon::from_name("find-location-symbolic").size(size).into()
}

pub fn icon_star_filled(size: u16) -> Icon {
    icon::from_name("starred-symbolic").size(size).into()
}

pub fn icon_star_outline(size: u16) -> Icon {
    icon::from_name("non-starred-symbolic").size(size).into()
}

pub fn icon_refresh(size: u16) -> Icon {
    icon::from_name("view-refresh-symbolic").size(size).into()
}
