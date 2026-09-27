// SPDX-License-Identifier: MPL-2.0

use crate::chart::{ChartItem, PrecipBarsChart, TempCurveChart};
use crate::config::{Config, SavedPlace};
use crate::fl;
use crate::icons;
use crate::map::{MapData, MapService};
use crate::map_widget::{MapCanvas, MapMessageCallback};
use crate::weather::{
    openweather_icon_to_symbolic, LocationSuggestion, WeatherBundle, WeatherService,
    DEFAULT_API_KEY,
};
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::widget::scrollable::{Direction, Scrollbar};
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{self, about::About, menu};
use cosmic::Task;
use std::collections::HashMap;
use std::sync::Arc;

const APP_ID: &str = "io.github.HimanshuSharma_prog.weather_for_cosmic";
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icon.png");
const DEFAULT_CITY: &str = "Bengaluru, IN";
const DEFAULT_LAT: f64 = 12.9701;
const DEFAULT_LON: f64 = 77.5636;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextPage {
    About,
}

impl Default for ContextPage {
    fn default() -> Self {
        Self::About
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    SetUnit(bool),
    SelectDay(usize),
    SearchInputChanged(String),
    SearchSubmit,
    SelectSuggestion(LocationSuggestion),
    CloseSuggestions,
    RefreshWeather,
    MapClicked(f64, f64),
    MapPanned(f64, f64),
    MapZoomIn,
    MapZoomOut,
    MapRecenter,
    LocationNameResolved(String),
    ToggleSaveCurrentLocation,
    SelectSavedPlace(SavedPlace),
    RemoveSavedPlace(usize),
    ToggleSavedPlacesMenu,
    NextHourly,
    PrevHourly,
    WeatherLoaded(Result<WeatherBundle, String>),
    SuggestionsLoaded(Result<Vec<LocationSuggestion>, String>),
    MapLoaded(Result<MapData, String>),
}

impl MapMessageCallback for Message {
    fn on_map_click(lat: f64, lon: f64) -> Self {
        Message::MapClicked(lat, lon)
    }

    fn on_map_pan(lat: f64, lon: f64) -> Self {
        Message::MapPanned(lat, lon)
    }

    fn on_zoom_in() -> Self {
        Message::MapZoomIn
    }

    fn on_zoom_out() -> Self {
        Message::MapZoomOut
    }
}

pub struct AppModel {
    core: cosmic::Core,
    context_page: ContextPage,
    about: About,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: Config,
    weather: Option<WeatherBundle>,
    map_data: Option<Arc<MapData>>,
    map_handle: Option<cosmic::iced::widget::image::Handle>,
    is_loading: bool,
    status_message: Option<String>,
    selected_day_index: usize,
    search_input: String,
    suggestions: Vec<LocationSuggestion>,
    saved_places: Vec<SavedPlace>,
    show_saved_places: bool,
    current_location_title: String,
    current_country: String,
    current_lat: f64,
    current_lon: f64,
    map_center_lat: f64,
    map_center_lon: f64,
    map_zoom: u8,
    map_is_loading: bool,
    hourly_offset: usize,
}

fn ensure_desktop_integration() {
    if let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) {
        let apps_dir = home.join(".local/share/applications");
        let icons_dir = home.join(".local/share/icons/hicolor");
        let desktop_file = apps_dir.join(format!("{APP_ID}.desktop"));

        let desktop_content = format!(
            "[Desktop Entry]\nName={}\nName[en]={}\nComment=Weather for COSMIC Desktop\nType=Application\nIcon={}\nExec=weather_for_cosmic %F\nTerminal=false\nStartupNotify=true\nCategories=COSMIC\nKeywords=COSMIC\n",
            crate::fl!("app-title"),
            crate::fl!("app-title"),
            APP_ID
        );

        let _ = std::fs::create_dir_all(&apps_dir);
        let should_write = match std::fs::read_to_string(&desktop_file) {
            Ok(existing) => existing != desktop_content,
            Err(_) => true,
        };
        if should_write {
            let _ = std::fs::write(&desktop_file, &desktop_content);
            let _ = std::fs::write(apps_dir.join("weather_for_cosmic.desktop"), &desktop_content);
            let _ = std::process::Command::new("update-desktop-database")
                .arg(&apps_dir)
                .status();
        }

        let icon_png = icons_dir.join(format!("512x512/apps/{APP_ID}.png"));
        if !icon_png.exists() {
            let _ = std::fs::create_dir_all(icons_dir.join("512x512/apps"));
            let _ = std::fs::write(&icon_png, APP_ICON);
            let _ = std::fs::write(icons_dir.join("512x512/apps/weather_for_cosmic.png"), APP_ICON);
            let _ = std::process::Command::new("gtk-update-icon-cache")
                .arg("-f")
                .arg("-t")
                .arg(&icons_dir)
                .status();
        }
    }
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        ensure_desktop_integration();

        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_raster_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .developers([("Himanshu Sharma", "https://sharmahimanshu.vercel.app/")])
            .links([
                (fl!("website"), "https://sharmahimanshu.vercel.app/"),
                (fl!("repository"), REPOSITORY),
            ])
            .license(env!("CARGO_PKG_LICENSE"));

        let mut config = cosmic_config::Config::new(APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((_errors, config)) => config,
            })
            .unwrap_or_default();

        let mut saved_places = config.get_saved_places();
        if saved_places.is_empty() {
            saved_places.push(SavedPlace {
                name: DEFAULT_CITY.to_string(),
                lat: format!("{DEFAULT_LAT:.4}"),
                lon: format!("{DEFAULT_LON:.4}"),
            });
            config.set_saved_places(&saved_places);
        }

        let initial_title = if !config.default_city.is_empty() {
            config.default_city.clone()
        } else {
            DEFAULT_CITY.to_string()
        };

        let initial_lat = config
            .default_lat
            .parse::<f64>()
            .unwrap_or(DEFAULT_LAT);
        let initial_lon = config
            .default_lon
            .parse::<f64>()
            .unwrap_or(DEFAULT_LON);

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            key_binds: HashMap::new(),
            config: config.clone(),
            weather: None,
            map_data: None,
            map_handle: None,
            is_loading: true,
            status_message: Some(format!("Loading weather for {initial_title}...")),
            selected_day_index: 0,
            search_input: String::new(),
            suggestions: Vec::new(),
            saved_places,
            show_saved_places: false,
            current_location_title: initial_title.clone(),
            current_country: String::new(),
            current_lat: initial_lat,
            current_lon: initial_lon,
            map_center_lat: initial_lat,
            map_center_lon: initial_lon,
            map_zoom: 7,
            map_is_loading: false,
            hourly_offset: 0,
        };

        let api_key = if app.config.api_key.is_empty() {
            DEFAULT_API_KEY.to_string()
        } else {
            app.config.api_key.clone()
        };

        let title_task = app.update_title();
        let weather_task = Task::perform(
            WeatherService::load_by_coords(
                initial_lat,
                initial_lon,
                initial_title.clone(),
                api_key,
            ),
            |result| cosmic::action::app(Message::WeatherLoaded(result)),
        );

        let map_task = Task::perform(
            MapService::fetch_map(
                initial_lat,
                initial_lon,
                7,
                initial_lat,
                initial_lon,
                initial_title,
            ),
            |result| cosmic::action::app(Message::MapLoaded(result)),
        );

        (app, title_task.chain(weather_task).chain(map_task))
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        // Keep header minimal so window controls and styling blend natively into COSMIC
        vec![]
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![
            menu::Tree::with_children(
                Element::from(widget::button::icon(widget::icon::from_name("open-menu-symbolic"))),
                menu::items(
                    &self.key_binds,
                    vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
                ),
            ),
        ]);

        vec![menu_bar.into()]
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
        })
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let spacing = cosmic::theme::spacing();
        let space_s = spacing.space_s;
        let space_m = spacing.space_m;

        let body_content: Element<'_, Self::Message> = if let Some(weather) = &self.weather {
            let top_bar = self.view_top_bar(space_s);
            let day_strip = self.view_day_strip(weather, space_s);
            let middle_section = self.view_middle_section(weather, space_s, space_m);
            let map_section = self.view_map_section(weather, space_s);

            widget::column::with_capacity(4)
                .push(top_bar)
                .push(day_strip)
                .push(middle_section)
                .push(map_section)
                .spacing(space_m)
                .width(Length::Fill)
                .into()
        } else if self.is_loading {
            let top_bar = self.view_top_bar(space_s);
            let spinner = widget::progress_bar::indeterminate_circular().size(56.0);
            let loader_view = widget::column::with_capacity(3)
                .push(cosmic::iced::widget::space::vertical().height(80))
                .push(spinner)
                .push(
                    widget::text(
                        self.status_message
                            .as_deref()
                            .unwrap_or("Loading forecast and weather data..."),
                    )
                    .size(15),
                )
                .spacing(space_m)
                .align_x(Horizontal::Center);

            widget::column::with_capacity(2)
                .push(top_bar)
                .push(loader_view)
                .spacing(space_m)
                .width(Length::Fill)
                .into()
        } else {
            let top_bar = self.view_top_bar(space_s);
            widget::column::with_capacity(4)
                .push(top_bar)
                .push(widget::text::title2("Unable to load weather"))
                .push(widget::text(
                    self.status_message
                        .as_deref()
                        .unwrap_or("Check network or OpenWeather API key."),
                ))
                .push(
                    widget::button::text("Retry")
                        .on_press(Message::RefreshWeather)
                        .padding([8, 16]),
                )
                .spacing(space_m)
                .align_x(Horizontal::Center)
                .into()
        };

        // Center content within max-width container and ensure auto-adjustment in maximize and minimize states
        let inner_card = widget::container(body_content)
            .width(Length::Fill)
            .max_width(1280);

        let centered_container = widget::container(inner_card)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .padding([space_s, space_m]);

        widget::scrollable(centered_container)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::LaunchUrl(url) => {
                if let Err(err) = open::that_detached(&url) {
                    eprintln!("failed to open {url:?}: {err}");
                }
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::SetUnit(use_fahrenheit) => {
                self.config.use_fahrenheit = use_fahrenheit;
                self.persist_config();
            }

            Message::SelectDay(idx) => {
                self.selected_day_index = idx;
            }

            Message::SearchInputChanged(value) => {
                self.search_input = value.clone();
                let trimmed = value.trim().to_string();

                if trimmed.len() < 2 {
                    self.suggestions.clear();
                    return Task::none();
                }

                let api_key = if self.config.api_key.is_empty() {
                    DEFAULT_API_KEY.to_string()
                } else {
                    self.config.api_key.clone()
                };

                return Task::perform(
                    WeatherService::search_locations(trimmed, api_key),
                    |result| cosmic::action::app(Message::SuggestionsLoaded(result)),
                );
            }

            Message::SearchSubmit => {
                let trimmed = self.search_input.trim().to_string();
                if trimmed.is_empty() {
                    return Task::none();
                }

                if !self.suggestions.is_empty() {
                    let top = self.suggestions.remove(0);
                    return Task::done(cosmic::action::app(Message::SelectSuggestion(top)));
                }

                self.suggestions.clear();
                self.is_loading = true;
                self.status_message = Some(format!("Searching for {trimmed}..."));

                let api_key = if self.config.api_key.is_empty() {
                    DEFAULT_API_KEY.to_string()
                } else {
                    self.config.api_key.clone()
                };

                return Task::perform(
                    WeatherService::search_locations(trimmed.clone(), api_key.clone()),
                    move |result| match result {
                        Ok(mut locs) if !locs.is_empty() => {
                            let top = locs.remove(0);
                            cosmic::action::app(Message::SelectSuggestion(top))
                        }
                        _ => cosmic::action::app(Message::WeatherLoaded(Err(format!(
                            "No place found matching '{trimmed}'"
                        )))),
                    },
                );
            }

            Message::SelectSuggestion(suggestion) => {
                self.suggestions.clear();
                let title = suggestion.display_title();
                self.search_input = title.clone();
                self.current_location_title = title.clone();
                self.current_country = suggestion.country.clone();
                self.current_lat = suggestion.lat;
                self.current_lon = suggestion.lon;
                self.map_center_lat = suggestion.lat;
                self.map_center_lon = suggestion.lon;

                self.persist_current_location();

                self.is_loading = true;
                self.status_message = Some(format!("Loading weather for {title}..."));

                let api_key = if self.config.api_key.is_empty() {
                    DEFAULT_API_KEY.to_string()
                } else {
                    self.config.api_key.clone()
                };

                let w_task = Task::perform(
                    WeatherService::load_by_coords(
                        suggestion.lat,
                        suggestion.lon,
                        title.clone(),
                        api_key,
                    ),
                    |result| cosmic::action::app(Message::WeatherLoaded(result)),
                );

                let m_task = Task::perform(
                    MapService::fetch_map(
                        self.map_center_lat,
                        self.map_center_lon,
                        self.map_zoom,
                        self.current_lat,
                        self.current_lon,
                        title,
                    ),
                    |result| cosmic::action::app(Message::MapLoaded(result)),
                );

                return self.update_title().chain(w_task).chain(m_task);
            }

            Message::CloseSuggestions => {
                self.suggestions.clear();
            }

            Message::RefreshWeather => {
                self.is_loading = true;
                self.status_message = Some(format!(
                    "Refreshing weather for {}...",
                    self.current_location_title
                ));

                let api_key = if self.config.api_key.is_empty() {
                    DEFAULT_API_KEY.to_string()
                } else {
                    self.config.api_key.clone()
                };

                let w_task = Task::perform(
                    WeatherService::load_by_coords(
                        self.current_lat,
                        self.current_lon,
                        self.current_location_title.clone(),
                        api_key,
                    ),
                    |result| cosmic::action::app(Message::WeatherLoaded(result)),
                );

                let m_task = Task::perform(
                    MapService::fetch_map(
                        self.map_center_lat,
                        self.map_center_lon,
                        self.map_zoom,
                        self.current_lat,
                        self.current_lon,
                        self.current_location_title.clone(),
                    ),
                    |result| cosmic::action::app(Message::MapLoaded(result)),
                );

                return w_task.chain(m_task);
            }

            Message::LocationNameResolved(name) => {
                self.current_location_title = name.clone();
                self.persist_current_location();
                if let Some(ref data) = self.map_data {
                    let mut new_data = (**data).clone();
                    new_data.location_name = name;
                    self.map_data = Some(Arc::new(new_data));
                }
                return self.update_title();
            }

            Message::ToggleSaveCurrentLocation => {
                let name = if self.current_location_title.is_empty() {
                    DEFAULT_CITY.to_string()
                } else {
                    self.current_location_title.clone()
                };
                let lat = format!("{:.4}", self.current_lat);
                let lon = format!("{:.4}", self.current_lon);

                if let Some(pos) = self.saved_places.iter().position(|p| {
                    p.name == name || (p.lat == lat && p.lon == lon)
                }) {
                    self.saved_places.remove(pos);
                } else {
                    self.saved_places.push(SavedPlace { name, lat, lon });
                }

                self.config.set_saved_places(&self.saved_places);
                self.persist_config();
            }

            Message::SelectSavedPlace(place) => {
                self.show_saved_places = false;
                self.suggestions.clear();
                self.current_location_title = place.name.clone();
                let lat = place.lat.parse::<f64>().unwrap_or(DEFAULT_LAT);
                let lon = place.lon.parse::<f64>().unwrap_or(DEFAULT_LON);
                self.current_lat = lat;
                self.current_lon = lon;
                self.map_center_lat = lat;
                self.map_center_lon = lon;

                self.persist_current_location();

                self.is_loading = true;
                self.status_message = Some(format!("Loading weather for {}...", place.name));

                let api_key = if self.config.api_key.is_empty() {
                    DEFAULT_API_KEY.to_string()
                } else {
                    self.config.api_key.clone()
                };

                let w_task = Task::perform(
                    WeatherService::load_by_coords(lat, lon, place.name.clone(), api_key),
                    |result| cosmic::action::app(Message::WeatherLoaded(result)),
                );

                let m_task = Task::perform(
                    MapService::fetch_map(
                        self.map_center_lat,
                        self.map_center_lon,
                        self.map_zoom,
                        self.current_lat,
                        self.current_lon,
                        place.name,
                    ),
                    |result| cosmic::action::app(Message::MapLoaded(result)),
                );

                return self.update_title().chain(w_task).chain(m_task);
            }

            Message::RemoveSavedPlace(idx) => {
                if idx < self.saved_places.len() {
                    self.saved_places.remove(idx);
                    self.config.set_saved_places(&self.saved_places);
                    self.persist_config();
                }
            }

            Message::ToggleSavedPlacesMenu => {
                self.show_saved_places = !self.show_saved_places;
            }

            Message::NextHourly => {
                if let Some(weather) = &self.weather {
                    let total = weather.hourly_strip.len();
                    let max_offset = total.saturating_sub(4);
                    self.hourly_offset = (self.hourly_offset + 2).min(max_offset);
                }
            }

            Message::PrevHourly => {
                self.hourly_offset = self.hourly_offset.saturating_sub(2);
            }

            Message::MapClicked(lat, lon) => {
                self.current_lat = lat;
                self.current_lon = lon;
                self.map_center_lat = lat;
                self.map_center_lon = lon;
                let loc_str = format!("{:.2}°, {:.2}°", lat, lon);
                self.current_location_title = loc_str.clone();

                self.persist_current_location();

                self.is_loading = true;
                self.status_message = Some("Loading weather for selected location...".to_string());

                let api_key = if self.config.api_key.is_empty() {
                    DEFAULT_API_KEY.to_string()
                } else {
                    self.config.api_key.clone()
                };

                let rev_key = api_key.clone();
                let reverse_task = Task::perform(
                    WeatherService::reverse_geocode(lat, lon, rev_key),
                    move |res| match res {
                        Some(name) => cosmic::action::app(Message::LocationNameResolved(name)),
                        None => cosmic::action::app(Message::LocationNameResolved(format!("{lat:.2}°, {lon:.2}°"))),
                    },
                );

                let w_task = Task::perform(
                    WeatherService::load_by_coords(lat, lon, loc_str.clone(), api_key),
                    |result| cosmic::action::app(Message::WeatherLoaded(result)),
                );

                let m_task = Task::perform(
                    MapService::fetch_map(
                        self.map_center_lat,
                        self.map_center_lon,
                        self.map_zoom,
                        self.current_lat,
                        self.current_lon,
                        loc_str,
                    ),
                    |result| cosmic::action::app(Message::MapLoaded(result)),
                );

                return reverse_task.chain(w_task).chain(m_task);
            }

            Message::MapPanned(new_lat, new_lon) => {
                if self.map_is_loading {
                    return Task::none();
                }
                self.map_center_lat = new_lat;
                self.map_center_lon = new_lon;
                self.map_is_loading = true;

                return Task::perform(
                    MapService::fetch_map(
                        self.map_center_lat,
                        self.map_center_lon,
                        self.map_zoom,
                        self.current_lat,
                        self.current_lon,
                        self.current_location_title.clone(),
                    ),
                    |result| cosmic::action::app(Message::MapLoaded(result)),
                );
            }

            Message::MapZoomIn => {
                if self.map_is_loading {
                    return Task::none();
                }
                if self.map_zoom < 13 {
                    self.map_zoom += 1;
                    self.map_is_loading = true;
                    return Task::perform(
                        MapService::fetch_map(
                            self.map_center_lat,
                            self.map_center_lon,
                            self.map_zoom,
                            self.current_lat,
                            self.current_lon,
                            self.current_location_title.clone(),
                        ),
                        |result| cosmic::action::app(Message::MapLoaded(result)),
                    );
                }
            }

            Message::MapZoomOut => {
                if self.map_is_loading {
                    return Task::none();
                }
                if self.map_zoom > 4 {
                    self.map_zoom -= 1;
                    self.map_is_loading = true;
                    return Task::perform(
                        MapService::fetch_map(
                            self.map_center_lat,
                            self.map_center_lon,
                            self.map_zoom,
                            self.current_lat,
                            self.current_lon,
                            self.current_location_title.clone(),
                        ),
                        |result| cosmic::action::app(Message::MapLoaded(result)),
                    );
                }
            }

            Message::MapRecenter => {
                if self.map_is_loading {
                    return Task::none();
                }
                self.map_center_lat = self.current_lat;
                self.map_center_lon = self.current_lon;
                self.map_is_loading = true;
                return Task::perform(
                    MapService::fetch_map(
                        self.current_lat,
                        self.current_lon,
                        self.map_zoom,
                        self.current_lat,
                        self.current_lon,
                        self.current_location_title.clone(),
                    ),
                    |result| cosmic::action::app(Message::MapLoaded(result)),
                );
            }

            Message::WeatherLoaded(result) => {
                self.is_loading = false;
                match result {
                    Ok(weather) => {
                        self.status_message = None;
                        self.current_location_title = weather.location_name.clone();
                        self.current_country = weather.country.clone();
                        self.current_lat = weather.lat;
                        self.current_lon = weather.lon;

                        self.persist_current_location();

                        self.weather = Some(weather);
                        self.selected_day_index = 0;
                        self.hourly_offset = 0;
                    }
                    Err(err) => {
                        self.status_message = Some(err);
                    }
                }
                return self.update_title();
            }

            Message::SuggestionsLoaded(result) => {
                if let Ok(suggestions) = result {
                    self.suggestions = suggestions;
                } else {
                    self.suggestions.clear();
                }
            }

            Message::MapLoaded(result) => {
                self.map_is_loading = false;
                if let Ok(data) = result {
                    let handle = cosmic::iced::widget::image::Handle::from_rgba(
                        data.width,
                        data.height,
                        data.rgba_bytes.clone(),
                    );
                    self.map_handle = Some(handle);
                    self.map_data = Some(Arc::new(data));
                }
            }
        }

        Task::none()
    }
}

impl AppModel {
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");
        if !self.current_location_title.is_empty() {
            window_title.push_str(" — ");
            window_title.push_str(&self.current_location_title);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    fn persist_current_location(&mut self) {
        self.config.default_city = self.current_location_title.clone();
        self.config.default_lat = format!("{:.4}", self.current_lat);
        self.config.default_lon = format!("{:.4}", self.current_lon);
        self.persist_config();
    }

    fn persist_config(&self) {
        if let Ok(context) = cosmic_config::Config::new(APP_ID, Config::VERSION) {
            if let Err(err) = self.config.write_entry(&context) {
                eprintln!("Failed to save cosmic config: {err}");
            }
        }
    }

    /// Top header bar inside the centered container:
    /// [Weather] [°C | °F]  ·········  [📍 Location ▾] [★] [🔄] [🔍 Search input with dropdown]
    fn view_top_bar<'a>(&'a self, space_s: u16) -> Element<'a, Message> {
        let title = widget::text::title2("Weather");

        let is_f = self.config.use_fahrenheit;
        let c_btn = if !is_f {
            widget::button::suggested("°C").padding([6, 12])
        } else {
            widget::button::text("°C")
                .on_press(Message::SetUnit(false))
                .padding([6, 12])
        };

        let f_btn = if is_f {
            widget::button::suggested("°F").padding([6, 12])
        } else {
            widget::button::text("°F")
                .on_press(Message::SetUnit(true))
                .padding([6, 12])
        };

        let unit_pill = widget::container(
            widget::row::with_capacity(2)
                .push(c_btn)
                .push(f_btn)
                .spacing(2)
                .align_y(Alignment::Center),
        )
        .padding(2);

        let left_row = widget::row::with_capacity(2)
            .push(title)
            .push(unit_pill)
            .spacing(space_s)
            .align_y(Alignment::Center);

        // Location button with popover for saved places
        let is_current_saved = self.saved_places.iter().any(|p| {
            p.name == self.current_location_title
                || (p.lat == format!("{:.4}", self.current_lat)
                    && p.lon == format!("{:.4}", self.current_lon))
        });

        let star_handle = if is_current_saved {
            icons::handle_star_filled(20)
        } else {
            icons::handle_star_outline(20)
        };

        let star_btn = widget::button::icon(star_handle)
            .on_press(Message::ToggleSaveCurrentLocation)
            .tooltip(if is_current_saved {
                "Remove from saved locations"
            } else {
                "Save this location"
            });

        let loc_content = widget::row::with_capacity(3)
            .push(icons::icon_location(18))
            .push(widget::text(&self.current_location_title).size(15))
            .push(widget::text("▾").size(12))
            .spacing(6)
            .align_y(Alignment::Center);

        let loc_btn = widget::button::custom(loc_content)
            .class(cosmic::theme::Button::Text)
            .on_press(Message::ToggleSavedPlacesMenu)
            .padding([6, 10]);

        let loc_widget: Element<'a, Message> = if self.show_saved_places {
            let saved_dropdown = self.view_saved_places_dropdown(space_s);
            widget::popover(loc_btn)
                .popup(saved_dropdown)
                .position(cosmic::widget::popover::Position::Bottom)
                .into()
        } else {
            loc_btn.into()
        };

        let refresh_btn = widget::button::icon(icons::handle_refresh(20))
            .on_press(Message::RefreshWeather)
            .tooltip("Refresh forecast");

        // Search box with integrated popover suggestions
        let search_box = widget::text_input("Search City or Zipcode...", &self.search_input)
            .on_input(Message::SearchInputChanged)
            .on_submit(|_| Message::SearchSubmit)
            .width(280)
            .padding([8, 12]);

        let search_widget: Element<'a, Message> = if !self.suggestions.is_empty() {
            let dropdown_content = self.view_suggestions_dropdown(space_s);
            widget::popover(search_box)
                .popup(dropdown_content)
                .position(cosmic::widget::popover::Position::Bottom)
                .into()
        } else {
            search_box.into()
        };

        let right_row = widget::row::with_capacity(4)
            .push(loc_widget)
            .push(star_btn)
            .push(refresh_btn)
            .push(search_widget)
            .spacing(space_s)
            .align_y(Alignment::Center);

        widget::row::with_capacity(3)
            .push(left_row)
            .push(cosmic::iced::widget::space::horizontal())
            .push(right_row)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    }

    /// Place suggestions dropdown - attached directly under the search input
    fn view_suggestions_dropdown<'a>(&'a self, space_s: u16) -> Element<'a, Message> {
        let mut list = widget::column::with_capacity(self.suggestions.len() + 1);

        let header = widget::row::with_capacity(3)
            .push(icons::icon_search(16))
            .push(widget::text("Locations").size(13))
            .push(cosmic::iced::widget::space::horizontal())
            .push(
                widget::button::icon(widget::icon::from_name("window-close-symbolic"))
                    .on_press(Message::CloseSuggestions)
                    .padding([2, 4]),
            )
            .spacing(8)
            .align_y(Alignment::Center);

        list = list.push(header);

        for suggestion in &self.suggestions {
            let label = suggestion.label();
            let btn_content = widget::row::with_capacity(2)
                .push(icons::icon_location(16))
                .push(widget::text(label).size(13))
                .spacing(8)
                .align_y(Alignment::Center);

            let btn = widget::button::custom(btn_content)
                .class(cosmic::theme::Button::Text)
                .on_press(Message::SelectSuggestion(suggestion.clone()))
                .padding([8, 10])
                .width(Length::Fill);

            list = list.push(btn);
        }

        widget::container(list.spacing(space_s / 3))
            .class(cosmic::theme::Container::Dropdown)
            .padding([space_s, space_s])
            .width(280)
            .into()
    }

    /// Saved locations dropdown - attached directly under the location badge
    fn view_saved_places_dropdown<'a>(&'a self, space_s: u16) -> Element<'a, Message> {
        let mut list = widget::column::with_capacity(self.saved_places.len() + 2);

        let header = widget::row::with_capacity(3)
            .push(icons::icon_star_filled(16))
            .push(widget::text("Saved Locations").size(13))
            .push(cosmic::iced::widget::space::horizontal())
            .push(
                widget::button::icon(widget::icon::from_name("window-close-symbolic"))
                    .on_press(Message::ToggleSavedPlacesMenu)
                    .padding([2, 4]),
            )
            .spacing(8)
            .align_y(Alignment::Center);

        list = list.push(header);

        for (idx, place) in self.saved_places.iter().enumerate() {
            let is_active = place.name == self.current_location_title;
            let icon = if is_active {
                icons::icon_star_filled(16)
            } else {
                icons::icon_location(16)
            };

            let btn_content = widget::row::with_capacity(2)
                .push(icon)
                .push(widget::text(&place.name).size(13))
                .spacing(8)
                .align_y(Alignment::Center);

            let place_btn = widget::button::custom(btn_content)
                .class(if is_active {
                    cosmic::theme::Button::Suggested
                } else {
                    cosmic::theme::Button::Text
                })
                .on_press(Message::SelectSavedPlace(place.clone()))
                .padding([8, 10])
                .width(Length::Fill);

            let del_btn = widget::button::icon(widget::icon::from_name("list-remove-symbolic"))
                .on_press(Message::RemoveSavedPlace(idx))
                .padding([4, 6])
                .tooltip("Remove location");

            let row = widget::row::with_capacity(2)
                .push(place_btn)
                .push(del_btn)
                .spacing(4)
                .align_y(Alignment::Center);

            list = list.push(row);
        }

        let is_current_saved = self.saved_places.iter().any(|p| {
            p.name == self.current_location_title
                || (p.lat == format!("{:.4}", self.current_lat)
                    && p.lon == format!("{:.4}", self.current_lon))
        });

        if !is_current_saved {
            let add_btn = widget::button::text("+ Save Current Location")
                .class(cosmic::theme::Button::Standard)
                .on_press(Message::ToggleSaveCurrentLocation)
                .padding([8, 12])
                .width(Length::Fill);

            list = list.push(add_btn);
        }

        widget::container(list.spacing(space_s / 2))
            .class(cosmic::theme::Container::Dropdown)
            .padding([space_s, space_s])
            .width(280)
            .into()
    }

    /// Horizontal row of 8 day pill cards directly below header
    fn view_day_strip<'a>(&'a self, weather: &'a WeatherBundle, space_s: u16) -> Element<'a, Message> {
        let mut row = widget::row::with_capacity(weather.daily_strip.len());
        let unit_str = if self.config.use_fahrenheit { "°F" } else { "°C" };

        for (idx, item) in weather.daily_strip.iter().enumerate() {
            let temp_val = self.format_temp(item.temp_c);
            let icon_sym = openweather_icon_to_symbolic(&item.icon);

            let is_selected = idx == self.selected_day_index;

            let content = widget::row::with_capacity(3)
                .push(widget::text(&item.day_label).size(14))
                .push(widget::text(format!("{temp_val}{unit_str}")).size(15))
                .push(widget::icon::from_name(icon_sym).size(20))
                .spacing(space_s / 2)
                .align_y(Alignment::Center);

            let pill = if is_selected {
                widget::button::custom(content)
                    .class(cosmic::theme::Button::Suggested)
                    .on_press(Message::SelectDay(idx))
                    .padding([8, 16])
            } else {
                widget::button::custom(content)
                    .class(cosmic::theme::Button::Standard)
                    .on_press(Message::SelectDay(idx))
                    .padding([8, 16])
            };

            row = row.push(pill);
        }

        let container = widget::container(
            widget::scrollable(row.spacing(space_s / 2))
                .direction(Direction::Horizontal(Scrollbar::default())),
        )
        .width(Length::Fill);

        container.into()
    }

    /// Middle section:
    /// Left column: Unified Card (Current Temp view + 6 Metric Cards merged into ONE card divided into 2 rows)
    /// Right column: Hourly forecast Card (Canvas curve + hourly cards with temp beside icon)
    fn view_middle_section<'a>(
        &'a self,
        weather: &'a WeatherBundle,
        space_s: u16,
        space_m: u16,
    ) -> Element<'a, Message> {
        let is_dark = cosmic::theme::active().cosmic().is_dark;
        let unit_str = if self.config.use_fahrenheit { "°F" } else { "°C" };

        // 1. LEFT COLUMN: Unified Card (Current Temp + 6 Metrics in 1 card divided into 2 rows)
        let cur_temp = self.format_temp(weather.temp_c);
        let feels_temp = self.format_temp(weather.feels_like_c);
        let icon_sym = openweather_icon_to_symbolic(&weather.icon);

        // Row 1 (Top row): Current temp and icon on the left side (Icon BEFORE temp, showing unit with temp)
        let weather_icon = widget::icon::from_name(icon_sym).size(52);
        let temp_text = widget::text(format!("{cur_temp}{unit_str}")).size(48);
        let desc_text = widget::text(&weather.description).size(16);
        let feels_text = widget::text(format!("Feels like {feels_temp}{unit_str}")).size(14);

        let temp_desc_col = widget::column::with_capacity(2)
            .push(desc_text)
            .push(feels_text)
            .spacing(2);

        let top_left = widget::row::with_capacity(3)
            .push(weather_icon)
            .push(temp_text)
            .push(temp_desc_col)
            .spacing(14)
            .align_y(Alignment::Center);

        // Top row - Right side: Time badge
        let time_badge = widget::row::with_capacity(2)
            .push(widget::icon::from_name("preferences-system-time-symbolic").size(16))
            .push(widget::text(&weather.local_time_str).size(14))
            .spacing(6)
            .align_y(Alignment::Center);

        let top_row = widget::row::with_capacity(3)
            .push(top_left)
            .push(cosmic::iced::widget::space::horizontal())
            .push(time_badge)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        // Row 2 (2nd row): The 6 metric cards (wind, humidity, visibility, pressure, uv index, dew point)
        let wind_tile = self.view_metric_tile(
            icons::icon_wind(20),
            "Wind",
            &format!("{:.0} m/s {}", weather.wind_speed_mps, weather.wind_direction),
            space_s,
        );
        let humidity_tile = self.view_metric_tile(
            icons::icon_humidity(20),
            "Humidity",
            &format!("{}%", weather.humidity),
            space_s,
        );
        let vis_tile = self.view_metric_tile(
            icons::icon_visibility(20),
            "Visibility",
            &format!("{:.0} km", weather.visibility_km),
            space_s,
        );
        let pressure_tile = self.view_metric_tile(
            icons::icon_pressure(20),
            "Pressure",
            &format!("{} hPa", weather.pressure_hpa),
            space_s,
        );
        let uv_tile = self.view_metric_tile(
            icons::icon_uv_index(20),
            "UV Index",
            &format!("{} UV", weather.uv_index),
            space_s,
        );
        let dew_tile = self.view_metric_tile(
            icons::icon_dew_point(20),
            "Dew Point",
            &format!("{}{unit_str}", self.format_temp(weather.dew_point_c)),
            space_s,
        );

        let metrics_subrow1 = widget::row::with_capacity(3)
            .push(widget::container(wind_tile).width(Length::FillPortion(1)))
            .push(widget::container(humidity_tile).width(Length::FillPortion(1)))
            .push(widget::container(vis_tile).width(Length::FillPortion(1)))
            .spacing(space_s);

        let metrics_subrow2 = widget::row::with_capacity(3)
            .push(widget::container(pressure_tile).width(Length::FillPortion(1)))
            .push(widget::container(uv_tile).width(Length::FillPortion(1)))
            .push(widget::container(dew_tile).width(Length::FillPortion(1)))
            .spacing(space_s);

        let row2_metrics = widget::column::with_capacity(2)
            .push(metrics_subrow1)
            .push(metrics_subrow2)
            .spacing(space_s);

        // Merge Row 1 and Row 2 into ONE SINGLE CARD:
        let unified_card_inner = widget::column::with_capacity(3)
            .push(top_row)
            .push(cosmic::iced::widget::space::vertical().height(space_s / 2))
            .push(row2_metrics)
            .spacing(space_s);

        let unified_card = widget::container(unified_card_inner)
            .class(cosmic::theme::Container::Card)
            .padding([space_m, space_m])
            .width(Length::Fill);

        // 2. RIGHT COLUMN: Hourly Forecast Card (Canvas curve + hourly cards with temp beside icon)
        let total_items = weather.hourly_strip.len();
        let visible_count = 4;
        let max_offset = total_items.saturating_sub(visible_count);
        let start_idx = self.hourly_offset.min(max_offset);
        let end_idx = (start_idx + visible_count).min(total_items);
        let visible_items = &weather.hourly_strip[start_idx..end_idx];

        let can_prev = start_idx > 0;
        let can_next = end_idx < total_items;

        let prev_btn = widget::button::icon(widget::icon::from_name("go-previous-symbolic").size(14))
            .on_press_maybe(can_prev.then_some(Message::PrevHourly))
            .padding([4, 8]);

        let next_btn = widget::button::icon(widget::icon::from_name("go-next-symbolic").size(14))
            .on_press_maybe(can_next.then_some(Message::NextHourly))
            .padding([4, 8]);

        let hourly_title = widget::text::title3("Hourly forecast");

        let hourly_header = widget::row::with_capacity(4)
            .push(hourly_title)
            .push(cosmic::iced::widget::space::horizontal())
            .push(prev_btn)
            .push(next_btn)
            .spacing(space_s / 2)
            .align_y(Alignment::Center);

        let chart_items: Vec<ChartItem> = visible_items
            .iter()
            .map(|item| ChartItem {
                temp: if self.config.use_fahrenheit {
                    (item.temp_c * 9.0 / 5.0) + 32.0
                } else {
                    item.temp_c
                },
            })
            .collect();

        let curve_chart = widget::canvas::Canvas::new(TempCurveChart::new(chart_items, is_dark))
            .width(Length::Fill)
            .height(100);

        let mut hourly_cards_row = widget::row::with_capacity(visible_items.len());
        for item in visible_items {
            let temp_str = format!("{}{unit_str}", self.format_temp(item.temp_c));
            let icon_sym = openweather_icon_to_symbolic(&item.icon);

            // Temp beside icon:
            let icon_and_temp = widget::row::with_capacity(2)
                .push(widget::icon::from_name(icon_sym).size(20))
                .push(widget::text(temp_str).size(14))
                .spacing(5)
                .align_y(Alignment::Center);

            // Raindrop icon + POP percentage:
            let pop_row = widget::row::with_capacity(2)
                .push(widget::icon::from_name("weather-showers-symbolic").size(12))
                .push(widget::text(&item.pop_percent).size(12))
                .spacing(4)
                .align_y(Alignment::Center);

            let card = widget::container(
                widget::column::with_capacity(3)
                    .push(widget::text(&item.time_label).size(13))
                    .push(icon_and_temp)
                    .push(pop_row)
                    .spacing(space_s / 3)
                    .align_x(Horizontal::Center),
            )
            .class(cosmic::theme::Container::Card)
            .padding([8, 10])
            .width(Length::FillPortion(1));

            hourly_cards_row = hourly_cards_row.push(card);
        }

        let hourly_cards_container = widget::container(
            hourly_cards_row.spacing(space_s / 2)
        )
        .width(Length::Fill);

        let right_card_inner = widget::column::with_capacity(3)
            .push(hourly_header)
            .push(curve_chart)
            .push(hourly_cards_container)
            .spacing(space_s);

        let right_column = widget::container(right_card_inner)
            .class(cosmic::theme::Container::Card)
            .padding([space_m, space_m])
            .width(Length::Fill);

        // Combined Middle Row: Left Column (Unified Card) and Right Column (Hourly Forecast Card)
        widget::row::with_capacity(2)
            .push(widget::container(unified_card).width(Length::FillPortion(1)))
            .push(widget::container(right_column).width(Length::FillPortion(1)))
            .spacing(space_m)
            .width(Length::Fill)
            .into()
    }

    /// Single metric tile container
    fn view_metric_tile<'a>(
        &'a self,
        icon: cosmic::widget::icon::Icon,
        title: &'static str,
        val: &str,
        space_s: u16,
    ) -> Element<'a, Message> {
        let content = widget::column::with_capacity(2)
            .push(
                widget::row::with_capacity(2)
                    .push(icon)
                    .push(widget::text(title).size(13))
                    .spacing(space_s / 2)
                    .align_y(Alignment::Center),
            )
            .push(widget::text(val.to_string()).size(16))
            .spacing(space_s / 4);

        widget::container(content)
            .class(cosmic::theme::Container::Card)
            .padding([10, 12])
            .width(Length::Fill)
            .into()
    }

    /// Bottom section: Interactive OpenStreetMap with Location Pin + Minute Precipitation Card Overlay
    fn view_map_section<'a>(&'a self, weather: &'a WeatherBundle, space_s: u16) -> Element<'a, Message> {
        let is_dark = cosmic::theme::active().cosmic().is_dark;

        let can_zoom_in = self.map_zoom < 13 && !self.map_is_loading;
        let can_zoom_out = self.map_zoom > 4 && !self.map_is_loading;

        let zoom_out_btn = widget::button::icon(widget::icon::from_name("zoom-out-symbolic").size(15))
            .on_press_maybe(can_zoom_out.then_some(Message::MapZoomOut))
            .padding([5, 8]);

        let zoom_in_btn = widget::button::icon(widget::icon::from_name("zoom-in-symbolic").size(15))
            .on_press_maybe(can_zoom_in.then_some(Message::MapZoomIn))
            .padding([5, 8]);

        let recenter_btn = widget::button::icon(widget::icon::from_name("zoom-fit-best-symbolic").size(15))
            .on_press_maybe((!self.map_is_loading).then_some(Message::MapRecenter))
            .padding([5, 8]);

        let zoom_badge = widget::container(
            widget::text(format!("Zoom {}", self.map_zoom)).size(12),
        )
        .padding([4, 8])
        .class(cosmic::theme::Container::Card);

        let mut map_header = widget::row::with_capacity(6)
            .push(widget::text::title3("Weather map & radar"))
            .push(cosmic::iced::widget::space::horizontal());

        if self.map_is_loading {
            map_header = map_header.push(widget::text("Updating map...").size(12));
        }

        map_header = map_header
            .push(recenter_btn)
            .push(zoom_out_btn)
            .push(zoom_badge)
            .push(zoom_in_btn)
            .spacing(space_s / 2)
            .align_y(Alignment::Center);

        // Interactive Canvas Map with Drag Pan & Location Pin
        let map_widget = widget::canvas::Canvas::new(MapCanvas::new(
            self.map_handle.clone(),
            self.map_data.clone(),
            is_dark,
        ))
        .width(Length::Fill)
        .height(340);

        let map_card_inner = widget::column::with_capacity(2)
            .push(map_header)
            .push(map_widget)
            .spacing(space_s);

        let map_card = widget::container(map_card_inner)
            .class(cosmic::theme::Container::Card)
            .padding([space_s, space_s])
            .width(Length::Fill);

        // Minute forecast - precipitation card
        let precip_title = widget::row::with_capacity(3)
            .push(widget::text::title3("Minute forecast - precipitation"))
            .push(cosmic::iced::widget::space::horizontal())
            .push(self.view_legend_strip(space_s))
            .align_y(Alignment::Center);

        // 5 Time pills
        let mut time_pills_row = widget::row::with_capacity(weather.precip_intervals.len());
        for p in &weather.precip_intervals {
            let pill_inner = widget::column::with_capacity(2)
                .push(widget::text(&p.label).size(11))
                .push(widget::text(&p.time_str).size(12))
                .spacing(2)
                .align_x(Horizontal::Center);

            let pill = widget::container(pill_inner)
                .class(cosmic::theme::Container::Card)
                .padding([6, 12])
                .width(Length::Fill);

            time_pills_row = time_pills_row.push(pill);
        }

        // 60 Precipitation bars ticker using canvas
        let bars_canvas = widget::canvas::Canvas::new(PrecipBarsChart::new(weather.precip_bars.clone()))
            .width(Length::Fill)
            .height(28);

        let precip_card_inner = widget::column::with_capacity(3)
            .push(precip_title)
            .push(time_pills_row.spacing(space_s / 2))
            .push(bars_canvas)
            .spacing(space_s);

        let precip_card = widget::container(precip_card_inner)
            .class(cosmic::theme::Container::Card)
            .padding([space_s, space_s])
            .width(Length::Fill);

        widget::column::with_capacity(2)
            .push(map_card)
            .push(precip_card)
            .spacing(space_s)
            .width(Length::Fill)
            .into()
    }

    /// Color legend strip for precipitation levels
    fn view_legend_strip<'a>(&'a self, space_s: u16) -> Element<'a, Message> {
        let colors = [
            (
                cosmic::iced::Color::from_rgb8(16, 185, 129),
                "0-0.5mm/h",
            ),
            (
                cosmic::iced::Color::from_rgb8(5, 150, 105),
                "0.5-2.5mm/h",
            ),
            (
                cosmic::iced::Color::from_rgb8(245, 158, 11),
                "2.5-7.5mm/h",
            ),
            (cosmic::iced::Color::from_rgb8(239, 68, 68), "7.5+mm/h"),
        ];

        let mut row = widget::row::with_capacity(colors.len() + 1);
        for (col, label) in colors {
            let dot = widget::canvas::Canvas::new(crate::chart::LegendBox::new(col))
                .width(12)
                .height(12);

            let item = widget::row::with_capacity(2)
                .push(dot)
                .push(widget::text(label).size(10))
                .spacing(4)
                .align_y(Alignment::Center);

            row = row.push(item);
        }

        row.spacing(space_s).align_y(Alignment::Center).into()
    }

    fn format_temp(&self, temp_c: f64) -> String {
        if self.config.use_fahrenheit {
            format!("{:.0}", (temp_c * 9.0 / 5.0) + 32.0)
        } else {
            format!("{:.0}", temp_c)
        }
    }
}
