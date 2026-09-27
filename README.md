<div align="center">

<img src="./resources/icons/hicolor/128x128/apps/icon.png" alt="Weather for COSMIC Logo" width="128" height="128" />

# Weather for COSMIC

**A modern, intuitive, and feature-rich weather forecast application built natively in Rust for the [COSMIC Desktop Environment](https://github.com/pop-os/cosmic-epoch).**

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![COSMIC](https://img.shields.io/badge/COSMIC-Desktop_Environment-3a67d7?logo=linux&logoColor=white)](https://system76.com/cosmic)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-blue.svg)](https://opensource.org/licenses/MPL-2.0)
[![Developer](https://img.shields.io/badge/Author-Himanshu_Sharma-teal)](https://sharmahimanshu.vercel.app/)

[Features](#-key-features) • [Installation & Running](#-getting-started) • [Architecture](#-tech-stack) • [Configuration](#-configuration) • [Author](#-author--portfolio)

</div>

---

## ✨ Overview

**Weather for COSMIC** is a fast, responsive, and beautifully designed weather application developed specifically for the System76 COSMIC Desktop. It combines live atmospheric telemetry, visual interactive graphs, satellite radar tile maps, and seamless location management inside a unified interface adhering strictly to COSMIC's design language and theme engine.

---

## 🚀 Key Features

### ⛅ Live Atmospheric Telemetry
- **Primary Conditions Header**: Real-time temperature display with high/low bounds, "feels like" metrics, and weather condition badges.
- **Toggleable Units**: Effortlessly switch between Celsius (°C) and Fahrenheit (°F) with persistent preferences.
- **6-Metric Deep Dive Dashboard**:
  - 💨 **Wind Speed & Bearing**: Real-time velocity with cardinal wind direction indicators.
  - 💧 **Relative Humidity**: Percentage and moisture saturation levels.
  - ⏱️ **Barometric Pressure**: Atmospheric pressure in hPa.
  - 👁️ **Visibility Range**: Distance visibility in kilometers.
  - ☀️ **UV Index**: Sunlight UV exposure index with risk classification.
  - 🌡️ **Dew Point**: Thermal dew condensation temperature.

### 📊 Interactive Hourly Charts & Carousel
- **Gradient Filled Temperature Curve**: Smooth cubic Bezier spline with rich color fills visualizing hourly temperature transitions.
- **Precipitation Probability Bars**: Hourly rain and snow probability bars.
- **Paginated Carousel**: Step forwards and backwards across hours with dedicated navigation arrows.

### 📅 7-Day Extended Forecast
- Comprehensive multi-day outlook featuring condition icons, expected highs and lows, and precipitation tendencies.

### 🗺️ Interactive Weather & Satellite Map
- **Carto OpenStreetMap Integration**: High-resolution, zero-friction open-source map tiles.
- **Smooth Navigation**: Pan across geographical regions and zoom in/out with responsive controls.
- **Click-to-Inspect**: Click anywhere on the globe to pinpoint coordinates and immediately fetch local forecasts.

### 🔍 Instant Geocoding & Places
- **Auto-Complete Search**: Real-time location search powered by geocoding suggestions.
- **Saved Places & Favorites**: Star your favorite cities and switch between them in seconds via the quick-access header menu.
- **Automatic Persistence**: Remembers your active location and preferences across application restarts.

### 🎨 Native COSMIC Integration
- Built with [libcosmic](https://github.com/pop-os/libcosmic) and [Iced](https://github.com/iced-rs/iced).
- Dynamically adapts to COSMIC System Light and Dark themes, accent colors, and custom container styling.
- Native Wayland client with custom high-DPI desktop icons and dock association.

---

## 🛠️ Tech Stack

| Component | Technology | Description |
| :--- | :--- | :--- |
| **Language** | Rust 2024 Edition | High performance, memory-safe systems programming |
| **GUI Toolkit** | `libcosmic` / `iced` | Native COSMIC / Wayland interface and wgpu rendering |
| **Weather API** | OpenWeatherMap | Global telemetry, hourly data, and geocoding |
| **Mapping Engine** | OpenStreetMap / Carto | Async tile fetching and custom canvas tile rendering |
| **Localization** | `i18n-embed` / Fluent | Dynamic multilingual localization support |
| **Automation** | `just` | Simple and reproducible task runner |

---

## 📦 Getting Started

### Prerequisites

Ensure you have the following installed:
- **Rust toolchain** (latest stable with 2024 edition support):
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **just command runner**:
  ```sh
  cargo install just
  ```
- **System build dependencies** (Fedora / Pop!_OS / Ubuntu):
  ```sh
  # For Fedora
  sudo dnf install gcc git wayland-devel libxkbcommon-devel openssl-devel gtk3-devel
  
  # For Pop!_OS / Ubuntu
  sudo apt install build-essential git libwayland-dev libxkbcommon-dev libssl-dev libgtk-3-dev
  ```

---

### Running in Development

To compile, register user desktop entries and icons, and start the app:

```sh
just run
```

### Useful Recipes

| Command | Action |
| :--- | :--- |
| `just build-release` | Compiles optimized release binary (`target/release/weather_for_cosmic`) |
| `just check` | Runs Clippy with strict linting rules |
| `just install` | Installs binary, desktop entries, and icons to system `/usr` prefix |
| `just vendor` | Packages dependencies into a local vendored tarball |
| `just clean` | Removes build artifacts |

---

## ⚙️ Configuration

Application settings are managed through `cosmic-config` and stored under:
```
~/.config/cosmic/io.github.pop_os.cosmic-app-template/v1/
```

- `default_city`: Last selected city name
- `default_lat` / `default_lon`: Coordinate center
- `use_fahrenheit`: Temperature unit preference
- `saved_locations`: List of saved bookmarks
- `api_key`: Optional custom OpenWeatherMap API key

---

## 🌐 Localization

Translations are powered by [Project Fluent](https://projectfluent.org/):
- Translation files are stored in [`i18n/`](./i18n).
- English localization is available at [`i18n/en/weather_for_cosmic.ftl`](./i18n/en/weather_for_cosmic.ftl).
- New languages can be added by creating a subdirectory with the corresponding ISO 639-1 code (e.g. `i18n/fr/`, `i18n/de/`, `i18n/hi/`).

---

## 👨‍💻 Author & Portfolio

**Himanshu Sharma**
- 🌐 **Portfolio & Website:** [https://sharmahimanshu.vercel.app/](https://sharmahimanshu.vercel.app/)
- 🐙 **GitHub:** [@HimanshuSharma-prog](https://github.com/HimanshuSharma-prog)
- 📧 **Email:** [sharmahimanshu1611@gmail.com](mailto:sharmahimanshu1611@gmail.com)

---

## 📄 License

This project is licensed under the Mozilla Public License 2.0 (MPL-2.0). See [LICENSE](./LICENSE) for details.
