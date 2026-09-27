// SPDX-License-Identifier: MPL-2.0

use cosmic::iced::mouse::Cursor;
use cosmic::iced::widget::canvas::{Cache, Geometry, Gradient, Path, Program, Stroke};
use cosmic::iced::{Color, Point, Rectangle};
use cosmic::widget::canvas::path::Builder;
use cosmic::{Renderer, Theme};

#[derive(Debug, Clone)]
pub struct ChartItem {
    pub temp: f64,
}

pub struct TempCurveChart {
    pub items: Vec<ChartItem>,
    #[allow(dead_code)]
    pub is_dark: bool,
    cache: Cache,
}

impl TempCurveChart {
    pub fn new(items: Vec<ChartItem>, is_dark: bool) -> Self {
        Self {
            items,
            is_dark,
            cache: Cache::new(),
        }
    }
}

impl<Message> Program<Message, Theme, Renderer> for TempCurveChart {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            if self.items.is_empty() {
                return;
            }

            let width = frame.width();
            let height = frame.height();
            let n = self.items.len();
            let col_width = width / (n as f32);

            let mut min_t = self.items[0].temp;
            let mut max_t = self.items[0].temp;
            for item in &self.items {
                if item.temp < min_t {
                    min_t = item.temp;
                }
                if item.temp > max_t {
                    max_t = item.temp;
                }
            }

            let range = if (max_t - min_t).abs() < 1.0 {
                4.0
            } else {
                (max_t - min_t) + 3.0
            };

            let top_pad = 16.0;
            let bottom_pad = 16.0;
            let chart_h = (height - top_pad - bottom_pad).max(10.0);

            let points: Vec<Point> = self
                .items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let x = (i as f32 + 0.5) * col_width;
                    let norm = (item.temp - min_t) / range;
                    let y = height - bottom_pad - (norm as f32 * chart_h);
                    Point::new(x, y)
                })
                .collect();

            if points.is_empty() {
                return;
            }

            // 1. Build and fill closed path under the temperature curve with vertical gradient
            let fill_path = Path::new(|builder: &mut Builder| {
                if let (Some(first), Some(last)) = (points.first(), points.last()) {
                    builder.move_to(Point::new(0.0, height));
                    builder.line_to(Point::new(0.0, first.y));
                    builder.line_to(*first);

                    for i in 0..(points.len() - 1) {
                        let p0 = points[i];
                        let p1 = points[i + 1];
                        let cx = (p0.x + p1.x) / 2.0;
                        builder.bezier_curve_to(
                            Point::new(cx, p0.y),
                            Point::new(cx, p1.y),
                            p1,
                        );
                    }

                    builder.line_to(Point::new(width, last.y));
                    builder.line_to(Point::new(width, height));
                    builder.close();
                }
            });

            let linear_gradient = cosmic::widget::canvas::gradient::Linear::new(
                Point::new(0.0, top_pad),
                Point::new(0.0, height),
            )
            .add_stop(0.0, Color::from_rgba(0.96, 0.55, 0.18, 0.45))
            .add_stop(0.45, Color::from_rgba(0.96, 0.55, 0.18, 0.22))
            .add_stop(1.0, Color::from_rgba(0.96, 0.55, 0.18, 0.01));

            frame.fill(&fill_path, Gradient::Linear(linear_gradient));

            // 2. Draw smooth stroke curve over the top edge
            let stroke_path = Path::new(|builder: &mut Builder| {
                if let (Some(first), Some(last)) = (points.first(), points.last()) {
                    builder.move_to(Point::new(0.0, first.y));
                    builder.line_to(*first);

                    for i in 0..(points.len() - 1) {
                        let p0 = points[i];
                        let p1 = points[i + 1];
                        let cx = (p0.x + p1.x) / 2.0;
                        builder.bezier_curve_to(
                            Point::new(cx, p0.y),
                            Point::new(cx, p1.y),
                            p1,
                        );
                    }

                    builder.line_to(Point::new(width, last.y));
                }
            });

            let stroke_color = Color::from_rgba(0.96, 0.55, 0.18, 0.95);
            frame.stroke(
                &stroke_path,
                Stroke::default().with_color(stroke_color).with_width(2.5),
            );

            // 3. Draw sleek circular point indicators at each hourly data point
            for p in &points {
                let dot_outer = Path::circle(*p, 4.0);
                frame.fill(&dot_outer, Color::from_rgba(0.96, 0.55, 0.18, 1.0));

                let dot_inner = Path::circle(*p, 2.0);
                let inner_color = if self.is_dark {
                    Color::from_rgb8(34, 40, 52)
                } else {
                    Color::WHITE
                };
                frame.fill(&dot_inner, inner_color);
            }
        });

        vec![geometry]
    }
}

pub struct PrecipBarsChart {
    pub bars: Vec<f32>,
    cache: Cache,
}

impl PrecipBarsChart {
    pub fn new(bars: Vec<f32>) -> Self {
        Self {
            bars,
            cache: Cache::new(),
        }
    }
}

impl<Message> Program<Message, Theme, Renderer> for PrecipBarsChart {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            if self.bars.is_empty() {
                return;
            }

            let width = frame.width();
            let height = frame.height();
            let n = self.bars.len().max(1);
            let bar_pitch = width / n as f32;
            let bar_w = (bar_pitch * 0.45).clamp(2.0, 5.0);

            for (i, &val) in self.bars.iter().enumerate() {
                let center_x = (i as f32 + 0.5) * bar_pitch;
                let bar_x = center_x - (bar_w / 2.0);

                let bar_h = (val * (height - 4.0)).clamp(6.0, height);
                let bar_y = height - bar_h;

                let color = if val <= 0.05 {
                    Color::from_rgba(0.70, 0.72, 0.76, 0.45)
                } else if val <= 0.25 {
                    Color::from_rgb8(34, 197, 94) // 0-0.5 mm/h
                } else if val <= 0.50 {
                    Color::from_rgb8(21, 128, 61) // 0.5-2.5 mm/h
                } else if val <= 0.75 {
                    Color::from_rgb8(234, 179, 8) // 2.5-7.5 mm/h
                } else {
                    Color::from_rgb8(220, 38, 38) // 7.5+ mm/h
                };

                let rect_path = Path::rounded_rectangle(
                    Point::new(bar_x, bar_y),
                    cosmic::iced::Size::new(bar_w, bar_h),
                    1.5.into(),
                );
                frame.fill(&rect_path, color);
            }
        });

        vec![geometry]
    }
}

pub struct LegendBox {
    pub color: Color,
    cache: Cache,
}

impl LegendBox {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            cache: Cache::new(),
        }
    }
}

impl<Message> Program<Message, Theme, Renderer> for LegendBox {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let rect = Path::rounded_rectangle(
                Point::ORIGIN,
                bounds.size(),
                2.0.into(),
            );
            frame.fill(&rect, self.color);
        });
        vec![geometry]
    }
}

