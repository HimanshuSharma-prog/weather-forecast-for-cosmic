// SPDX-License-Identifier: MPL-2.0
#![allow(dead_code)]

use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct MapData {
    pub rgba_bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub zoom: u8,
    pub center_lat: f64,
    pub center_lon: f64,
    pub marker_lat: f64,
    pub marker_lon: f64,
    pub location_name: String,
    pub origin_x: f64,
    pub origin_y: f64,
    pub center_img_x: f64,
    pub center_img_y: f64,
    pub marker_img_x: f64,
    pub marker_img_y: f64,
    pub total_pixels: f64,
}

impl MapData {
    /// Calculate the screen position (x, y) of the marker on the canvas
    /// Calculate the rectangle where the map image should be drawn on canvas
    pub fn image_screen_rect(
        &self,
        display_w: f32,
        display_h: f32,
        drag_dx: f32,
        drag_dy: f32,
    ) -> (f32, f32, f32, f32) {
        let mut x = (display_w as f64 / 2.0) - self.center_img_x + drag_dx as f64;
        let mut y = (display_h as f64 / 2.0) - self.center_img_y + drag_dy as f64;

        let w = self.width as f64;
        let h = self.height as f64;
        let dw = display_w as f64;
        let dh = display_h as f64;

        // Ensure the map covers the entire display area without blank gaps on sides or edges
        if w >= dw {
            x = x.clamp(dw - w, 0.0);
        }
        if h >= dh {
            y = y.clamp(dh - h, 0.0);
        }

        (x as f32, y as f32, self.width as f32, self.height as f32)
    }

    /// Calculate the screen position (x, y) of the marker on the canvas
    pub fn marker_screen_pos(
        &self,
        display_w: f32,
        display_h: f32,
        drag_dx: f32,
        drag_dy: f32,
    ) -> (f32, f32) {
        let (ix, iy, _, _) = self.image_screen_rect(display_w, display_h, drag_dx, drag_dy);
        let mx = ix as f64 + self.marker_img_x;
        let my = iy as f64 + self.marker_img_y;
        (mx as f32, my as f32)
    }

    /// Convert a screen click coordinate (click_x, click_y) to geographic (lat, lon)
    pub fn pixel_to_coords(
        &self,
        click_x: f32,
        click_y: f32,
        display_w: f32,
        display_h: f32,
        drag_dx: f32,
        drag_dy: f32,
    ) -> (f64, f64) {
        let (img_dest_x, img_dest_y, _, _) =
            self.image_screen_rect(display_w, display_h, drag_dx, drag_dy);

        let img_click_x = click_x as f64 - img_dest_x as f64;
        let img_click_y = click_y as f64 - img_dest_y as f64;

        let world_x = self.origin_x + img_click_x;
        let world_y = self.origin_y + img_click_y;

        let lon = (world_x / self.total_pixels) * 360.0 - 180.0;
        let n = PI - 2.0 * PI * (world_y / self.total_pixels);
        let lat = n.sinh().atan().to_degrees();

        (lat.clamp(-85.0511, 85.0511), lon.clamp(-180.0, 180.0))
    }

    /// Convert a pan displacement (-drag_dx, -drag_dy) to new center (lat, lon)
    pub fn pan_to_coords(&self, pan_dx: f32, pan_dy: f32) -> (f64, f64) {
        let center_world_x = self.origin_x + self.center_img_x + pan_dx as f64;
        let center_world_y = self.origin_y + self.center_img_y + pan_dy as f64;

        let lon = (center_world_x / self.total_pixels) * 360.0 - 180.0;
        let n = PI - 2.0 * PI * (center_world_y / self.total_pixels);
        let lat = n.sinh().atan().to_degrees();

        (lat.clamp(-85.0511, 85.0511), lon.clamp(-180.0, 180.0))
    }
}

fn get_shared_client() -> reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .user_agent("WeatherForCosmic/0.1 (https://github.com/pop-os/weather_for_cosmic)")
                .build()
                .unwrap_or_default()
        })
        .clone()
}

fn get_cached_tile(z: u8, x: i32, y: i32) -> Option<Vec<u8>> {
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<(u8, i32, i32), Vec<u8>>>,
    > = std::sync::OnceLock::new();
    let map = CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    if let Ok(guard) = map.lock() {
        guard.get(&(z, x, y)).cloned()
    } else {
        None
    }
}

fn put_cached_tile(z: u8, x: i32, y: i32, bytes: Vec<u8>) {
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<(u8, i32, i32), Vec<u8>>>,
    > = std::sync::OnceLock::new();
    let map = CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    if let Ok(mut guard) = map.lock() {
        if guard.len() > 400 {
            guard.clear();
        }
        guard.insert((z, x, y), bytes);
    }
}

pub struct MapService;

impl MapService {
    pub async fn fetch_map(
        center_lat: f64,
        center_lon: f64,
        zoom: u8,
        marker_lat: f64,
        marker_lon: f64,
        location_name: String,
    ) -> Result<MapData, String> {
        let zoom = zoom.clamp(4, 13);
        let n = (1 << zoom) as f64;
        let total_pixels = n * 256.0;

        let x_center = ((center_lon + 180.0) / 360.0) * total_pixels;
        let lat_rad = center_lat.to_radians();
        let y_center =
            ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / PI) / 2.0) * total_pixels;

        let center_tile_x = (x_center / 256.0).floor() as i32;
        let center_tile_y = (y_center / 256.0).floor() as i32;

        let cols = 6;
        let rows = 3;
        let min_tile_x = center_tile_x - 3;
        let min_tile_y = center_tile_y - 1;

        let origin_x = min_tile_x as f64 * 256.0;
        let origin_y = min_tile_y as f64 * 256.0;

        let center_img_x = x_center - origin_x;
        let center_img_y = y_center - origin_y;

        // Marker position
        let marker_x_world = ((marker_lon + 180.0) / 360.0) * total_pixels;
        let marker_lat_rad = marker_lat.to_radians();
        let marker_y_world =
            ((1.0 - (marker_lat_rad.tan() + 1.0 / marker_lat_rad.cos()).ln() / PI) / 2.0)
                * total_pixels;

        let marker_img_x = marker_x_world - origin_x;
        let marker_img_y = marker_y_world - origin_y;

        let client = get_shared_client();
        let mut img = image::RgbaImage::new((cols * 256) as u32, (rows * 256) as u32);

        // Fill background with light neutral map tone
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([232, 236, 241, 255]);
        }

        let max_tile = (1 << zoom) - 1;

        // Fetch free, open-source OpenStreetMap tiles in parallel with caching
        let mut fetch_tasks = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                let tile_x = (min_tile_x + c).rem_euclid(1 << zoom);
                let tile_y = (min_tile_y + r).clamp(0, max_tile);

                // Check cache first
                if let Some(cached_bytes) = get_cached_tile(zoom, tile_x, tile_y) {
                    if let Ok(tile_img) = image::load_from_memory(&cached_bytes) {
                        image::imageops::overlay(&mut img, &tile_img.to_rgba8(), (c * 256) as i64, (r * 256) as i64);
                        continue;
                    }
                }

                let primary_url = format!("https://tile.openstreetmap.org/{zoom}/{tile_x}/{tile_y}.png");
                let fallback_url = format!("https://tile.openstreetmap.de/{zoom}/{tile_x}/{tile_y}.png");
                let client_ref = client.clone();

                fetch_tasks.push(async move {
                    // Try primary OpenStreetMap server
                    if let Ok(resp) = client_ref.get(&primary_url).send().await {
                        if resp.status().is_success() {
                            if let Ok(bytes) = resp.bytes().await {
                                if let Ok(tile_img) = image::load_from_memory(&bytes) {
                                    put_cached_tile(zoom, tile_x, tile_y, bytes.to_vec());
                                    return Some((c as u32, r as u32, tile_img.to_rgba8()));
                                }
                            }
                        }
                    }
                    // Try fallback OpenStreetMap server
                    if let Ok(resp) = client_ref.get(&fallback_url).send().await {
                        if resp.status().is_success() {
                            if let Ok(bytes) = resp.bytes().await {
                                if let Ok(tile_img) = image::load_from_memory(&bytes) {
                                    put_cached_tile(zoom, tile_x, tile_y, bytes.to_vec());
                                    return Some((c as u32, r as u32, tile_img.to_rgba8()));
                                }
                            }
                        }
                    }
                    None
                });
            }
        }

        if !fetch_tasks.is_empty() {
            let results = futures::future::join_all(fetch_tasks).await;
            for item in results.into_iter().flatten() {
                let (c, r, tile_rgba) = item;
                let dest_x = c * 256;
                let dest_y = r * 256;
                image::imageops::overlay(&mut img, &tile_rgba, dest_x as i64, dest_y as i64);
            }
        }

        Ok(MapData {
            rgba_bytes: img.into_raw(),
            width: (cols * 256) as u32,
            height: (rows * 256) as u32,
            zoom,
            center_lat,
            center_lon,
            marker_lat,
            marker_lon,
            location_name,
            origin_x,
            origin_y,
            center_img_x,
            center_img_y,
            marker_img_x,
            marker_img_y,
            total_pixels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_pixel_to_coords() {
        let zoom = 7;
        let orig_lat: f64 = 12.9716;
        let orig_lon: f64 = 77.5946;

        let n = (1 << zoom) as f64;
        let total_pixels = n * 256.0;

        let x_center = ((orig_lon + 180.0) / 360.0) * total_pixels;
        let lat_rad = orig_lat.to_radians();
        let y_center =
            ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / PI) / 2.0) * total_pixels;

        let center_tile_x = (x_center / 256.0).floor() as i32;
        let center_tile_y = (y_center / 256.0).floor() as i32;

        let cols = 5;
        let rows = 3;
        let min_tile_x = center_tile_x - 2;
        let min_tile_y = center_tile_y - 1;

        let origin_x = min_tile_x as f64 * 256.0;
        let origin_y = min_tile_y as f64 * 256.0;

        let center_img_x = x_center - origin_x;
        let center_img_y = y_center - origin_y;

        let data = MapData {
            rgba_bytes: Vec::new(),
            width: (cols * 256) as u32,
            height: (rows * 256) as u32,
            zoom,
            center_lat: orig_lat,
            center_lon: orig_lon,
            marker_lat: orig_lat,
            marker_lon: orig_lon,
            location_name: "Bengaluru".to_string(),
            origin_x,
            origin_y,
            center_img_x,
            center_img_y,
            marker_img_x: center_img_x,
            marker_img_y: center_img_y,
            total_pixels,
        };

        let display_w = 800.0;
        let display_h = 340.0;

        let (mx, my) = data.marker_screen_pos(display_w, display_h, 0.0, 0.0);
        assert!((mx - display_w / 2.0).abs() < 1.0);
        assert!((my - display_h / 2.0).abs() < 1.0);

        let (calc_lat, calc_lon) =
            data.pixel_to_coords(mx, my, display_w, display_h, 0.0, 0.0);

        assert!((calc_lat - orig_lat).abs() < 0.01);
        assert!((calc_lon - orig_lon).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_fetch_osm_tile() {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .user_agent("WeatherForCosmic/0.1 (https://github.com/pop-os/weather_for_cosmic)")
            .build()
            .unwrap();
        let url = "https://a.tile.openstreetmap.fr/osmfr/7/64/64.png";
        let resp = client.get(url).send().await.unwrap();
        assert!(resp.status().is_success());
        let bytes = resp.bytes().await.unwrap();
        let img = image::load_from_memory(&bytes).unwrap();
        assert_eq!(img.width(), 256);
        assert_eq!(img.height(), 256);
        println!("Successfully fetched & decoded OSM tile: 256x256");
    }
}
