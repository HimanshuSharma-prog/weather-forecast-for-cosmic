// SPDX-License-Identifier: MPL-2.0

use crate::map::MapData;
use cosmic::iced::mouse::{self, Cursor};
use cosmic::iced::widget::canvas::{self, Action, Event, Frame, Geometry, Path, Program, Stroke};
use cosmic::iced::{Color, Point, Rectangle, Size};
use cosmic::widget::canvas::path::Builder;
use cosmic::{Renderer, Theme};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct MapCanvasState {
    pub is_dragging: bool,
    pub drag_start: Option<Point>,
    pub current_cursor: Option<Point>,
    pub drag_offset: (f32, f32),
}

pub struct MapCanvas {
    pub map_handle: Option<cosmic::iced::widget::image::Handle>,
    pub map_data: Option<Arc<MapData>>,
    pub is_dark: bool,
}

impl MapCanvas {
    pub fn new(
        map_handle: Option<cosmic::iced::widget::image::Handle>,
        map_data: Option<Arc<MapData>>,
        is_dark: bool,
    ) -> Self {
        Self {
            map_handle,
            map_data,
            is_dark,
        }
    }
}

#[allow(dead_code)]
pub trait MapMessageCallback: Clone + 'static {
    fn on_map_click(lat: f64, lon: f64) -> Self;
    fn on_map_pan(lat: f64, lon: f64) -> Self;
    fn on_zoom_in() -> Self;
    fn on_zoom_out() -> Self;
}

impl<Message> Program<Message, Theme, Renderer> for MapCanvas
where
    Message: MapMessageCallback,
{
    type State = MapCanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    state.is_dragging = true;
                    state.drag_start = Some(pos);
                    state.current_cursor = Some(pos);
                    state.drag_offset = (0.0, 0.0);
                }
            }

            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    state.current_cursor = Some(pos);
                    if state.is_dragging {
                        if let Some(start) = state.drag_start {
                            state.drag_offset = (pos.x - start.x, pos.y - start.y);
                        }
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.is_dragging {
                    state.is_dragging = false;
                    let (dx, dy) = state.drag_offset;
                    state.drag_offset = (0.0, 0.0);
                    let dist = dx.hypot(dy);

                    if dist < 6.0 {
                        // Click event: resolve clicked coordinates
                        if let Some(pos) = state.drag_start {
                            if let Some(map_data) = &self.map_data {
                                let (lat, lon) = map_data.pixel_to_coords(
                                    pos.x,
                                    pos.y,
                                    bounds.width,
                                    bounds.height,
                                    0.0,
                                    0.0,
                                );
                                return Some(Action::publish(Message::on_map_click(lat, lon)));
                            }
                        }
                    } else if dist >= 20.0 {
                        // Pan event: move center by drag offset
                        if let Some(map_data) = &self.map_data {
                            let (new_lat, new_lon) = map_data.pan_to_coords(-dx, -dy);
                            return Some(Action::publish(Message::on_map_pan(new_lat, new_lon)));
                        }
                    }
                }
            }

            _ => {}
        }
        None
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let width = bounds.width;
        let height = bounds.height;

        let (drag_x, drag_y) = state.drag_offset;

        // GEOMETRY 1: Map Tile Image Layer (rendered first)
        let mut map_frame = Frame::new(renderer, bounds.size());

        if let (Some(handle), Some(data)) = (&self.map_handle, &self.map_data) {
            let (ix, iy, iw, ih) = data.image_screen_rect(width, height, drag_x, drag_y);
            map_frame.draw_image(
                Rectangle {
                    x: ix,
                    y: iy,
                    width: iw,
                    height: ih,
                },
                handle,
            );
        } else {
            // Background fallback when tiles are loading
            let bg_color = Color::from_rgb8(18, 24, 32);
            let bg_path = Path::rounded_rectangle(Point::ORIGIN, Size::new(width, height), 16.0.into());
            map_frame.fill(&bg_path, bg_color);

            map_frame.fill_text(canvas::Text {
                content: "Loading OpenStreetMap tiles...".to_string(),
                position: Point::new(width / 2.0, height / 2.0),
                color: Color::from_rgb8(180, 190, 205),
                size: 14.0.into(),
                align_x: cosmic::iced::alignment::Horizontal::Center.into(),
                align_y: cosmic::iced::alignment::Vertical::Center.into(),
                ..Default::default()
            });
        }

        let map_geometry = map_frame.into_geometry();

        // GEOMETRY 2: Vector Overlay Layer (Pin, Badge, Controls, Rounded Border)
        // This is guaranteed to be rendered ON TOP of the map image!
        let mut pin_frame = Frame::new(renderer, bounds.size());

        if let Some(ref data) = self.map_data {
            let (mx, my) = data.marker_screen_pos(width, height, drag_x, drag_y);

            // Draw pin if it's within or near visible screen area
            if mx >= -40.0 && mx <= width + 40.0 && my >= -40.0 && my <= height + 40.0 {
                // Ground drop shadow under pin apex tip
                let shadow = Path::circle(Point::new(mx, my + 1.5), 5.5);
                pin_frame.fill(&shadow, Color::from_rgba(0.0, 0.0, 0.0, 0.38));

                // Teardrop Pin body:
                let head_center_y = my - 24.0;
                let head_radius = 12.0;

                let pin_path = Path::new(|builder: &mut Builder| {
                    builder.move_to(Point::new(mx, my)); // apex tip
                    builder.bezier_curve_to(
                        Point::new(mx - 4.5, my - 8.0),
                        Point::new(mx - head_radius, head_center_y + 6.0),
                        Point::new(mx - head_radius, head_center_y),
                    );
                    builder.bezier_curve_to(
                        Point::new(mx - head_radius, head_center_y - head_radius * 1.33),
                        Point::new(mx + head_radius, head_center_y - head_radius * 1.33),
                        Point::new(mx + head_radius, head_center_y),
                    );
                    builder.bezier_curve_to(
                        Point::new(mx + head_radius, head_center_y + 6.0),
                        Point::new(mx + 4.5, my - 8.0),
                        Point::new(mx, my),
                    );
                    builder.close();
                });

                // Vibrant Google Maps / OpenStreetMap style red pin
                pin_frame.fill(&pin_path, Color::from_rgb8(234, 67, 53));

                // Crisp white outer stroke
                pin_frame.stroke(
                    &pin_path,
                    Stroke::default()
                        .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.95))
                        .with_width(2.0),
                );

                // Inner white circle inside pin head
                let inner_dot = Path::circle(Point::new(mx, head_center_y), 4.5);
                pin_frame.fill(&inner_dot, Color::WHITE);

                // Location badge tag above the pin
                let tag_pad_h = 12.0;
                let char_w = 7.0;
                let tag_w =
                    (data.location_name.len() as f32 * char_w + tag_pad_h * 2.0).clamp(70.0, 240.0);
                let tag_h = 24.0;
                let tag_x = (mx - (tag_w / 2.0)).clamp(6.0, (width - tag_w - 6.0).max(6.0));
                let tag_y = (head_center_y - head_radius - tag_h - 4.0).max(6.0);

                let tag_rect = Path::rounded_rectangle(
                    Point::new(tag_x, tag_y),
                    Size::new(tag_w, tag_h),
                    6.0.into(),
                );

                // Tag drop shadow
                let tag_shadow = Path::rounded_rectangle(
                    Point::new(tag_x, tag_y + 1.5),
                    Size::new(tag_w, tag_h),
                    6.0.into(),
                );
                pin_frame.fill(&tag_shadow, Color::from_rgba(0.0, 0.0, 0.0, 0.30));

                pin_frame.fill(
                    &tag_rect,
                    if self.is_dark {
                        Color::from_rgba(0.12, 0.14, 0.18, 0.94)
                    } else {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.94)
                    },
                );

                pin_frame.stroke(
                    &tag_rect,
                    Stroke::default()
                        .with_color(if self.is_dark {
                            Color::from_rgba(1.0, 1.0, 1.0, 0.20)
                        } else {
                            Color::from_rgba(0.0, 0.0, 0.0, 0.15)
                        })
                        .with_width(1.0),
                );

                pin_frame.fill_text(canvas::Text {
                    content: data.location_name.clone(),
                    position: Point::new(tag_x + tag_w / 2.0, tag_y + tag_h / 2.0),
                    color: if self.is_dark {
                        Color::WHITE
                    } else {
                        Color::from_rgb8(20, 20, 25)
                    },
                    size: 11.5.into(),
                    align_x: cosmic::iced::alignment::Horizontal::Center.into(),
                    align_y: cosmic::iced::alignment::Vertical::Center.into(),
                    ..Default::default()
                });
            }
        }

        // Map info pill in bottom-left
        let info_rect = Path::rounded_rectangle(
            Point::new(10.0, height - 26.0),
            Size::new(200.0, 18.0),
            4.0.into(),
        );
        pin_frame.fill(&info_rect, Color::from_rgba(0.0, 0.0, 0.0, 0.65));
        pin_frame.fill_text(canvas::Text {
            content: "© OpenStreetMap contributors".to_string(),
            position: Point::new(110.0, height - 17.0),
            color: Color::from_rgb8(230, 235, 245),
            size: 10.0.into(),
            align_x: cosmic::iced::alignment::Horizontal::Center.into(),
            align_y: cosmic::iced::alignment::Vertical::Center.into(),
            ..Default::default()
        });

        // Crisp rounded border stroke around the entire map view (radius 12.0)
        let outer_border = Path::rounded_rectangle(
            Point::new(1.0, 1.0),
            Size::new(width - 2.0, height - 2.0),
            12.0.into(),
        );
        pin_frame.stroke(
            &outer_border,
            Stroke::default()
                .with_color(if self.is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.12)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.10)
                })
                .with_width(1.5),
        );

        let pin_geometry = pin_frame.into_geometry();

        vec![map_geometry, pin_geometry]
    }
}
