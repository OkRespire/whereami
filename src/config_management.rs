use iced::{Color, Theme, theme::Palette};
use serde::Deserialize;
use std::{
    fs,
    io::{self, Write},
};

/// Collects the toml file into an easy class.
/// Its contents are quite self-explanitory
#[derive(Deserialize)]
pub struct Config {
    pub theme: Option<String>,
    pub window: Window,
    pub colors: Colors,
    pub font: Font,
    pub layout: Layout,
    pub behavior: Behavior,
}

#[derive(Deserialize)]
pub struct Window {
    pub width: f32,
    pub height: f32,
    pub transparent: bool,
    pub decorations: bool,
}

/// NOTE: Selected text *may* not be used... keeping it in because it could be useful
#[derive(Deserialize)]
pub struct Colors {
    pub background: String,
    pub text: String,
    pub selected_background: String,
    pub selected_text: String,
    pub status: ColorsStatus,
}

#[derive(Deserialize)]
pub struct ColorsStatus {
    pub fullscreen: String,
    pub maximized: String,
    pub floating: String,
    pub tiled: String,
}

#[derive(Deserialize)]
pub struct Font {
    pub size: f32,
    // family: Option<String>,  // optional field
}

#[derive(Deserialize)]
pub struct Layout {
    pub padding: f32,
    pub margin: f32,
    pub spacing: f32,
    pub border_radius: f32,
}

/// Still need to implement all of this...
#[derive(Deserialize)]
pub struct Behavior {
    pub wrap_navigation: bool,
    pub refresh_interval: u64,
}

impl Default for Config {
    /// this defaults to gruvbox if no config file is found
    fn default() -> Self {
        Config {
            theme: None,
            window: Window {
                width: 600.0,
                height: 400.0,
                transparent: true,
                decorations: false,
            },
            colors: Colors {
                background: "#282828".to_string(),
                text: "#ebdbb2".to_string(),
                selected_background: "#458588".to_string(),
                selected_text: "#282828".to_string(),
                status: ColorsStatus {
                    fullscreen: "#fb4934".to_string(),
                    maximized: "#fabd2f".to_string(),
                    floating: "#b8bb26".to_string(),
                    tiled: "#83a598".to_string(),
                },
            },
            font: Font { size: 14.0 },
            layout: Layout {
                padding: 10.0,
                margin: 10.0,
                spacing: 5.0,
                border_radius: 4.0,
            },
            behavior: Behavior {
                wrap_navigation: true,
                refresh_interval: 1,
            },
        }
    }
}
impl Config {
    pub fn new() -> io::Result<Self> {
        let home = std::env::var("HOME").expect("HOME not set");

        let config_path = format!("{}/.config/whereami/config.toml", home);

        if std::path::Path::new(&config_path).exists() {
            let file_contents = fs::read_to_string(config_path)?;
            let config: Config = toml::from_str(&file_contents)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(config)
        } else {
            Self::create_config()?;
            Ok(Config::default())
        }
    }
    /// the constructor except you are writing it into ~/.config/whereami/config.toml
    pub fn create_config() -> io::Result<()> {
        let home = std::env::var("HOME").expect("HOME not set");

        let config_dir = format!("{}/.config/whereami", home);

        fs::create_dir_all(&config_dir)?;

        let mut file = fs::File::create(format!("{}/config.toml", &config_dir))?;

        let config_content = b"# Uncomment if you want a custom theme.
            # All list of themes in iced docs (https://docs.rs/iced/latest/iced/theme/enum.Theme.html)
            # theme = \"GruvboxDark\"

            [window]
            width = 600
            height = 400
            transparent = true
            decorations = false

            [colors]
            background = \"#282828\"
            text = \"#ebdbb2\"
            selected_background = \"#458588\"
            selected_text = \"#282828\"

            [colors.status]
            fullscreen = \"#fb4934\"
            maximized = \"#fabd2f\"
            floating = \"#b8bb26\"
            tiled = \"#83a598\"

            [font]
            size = 14

            [layout]
            padding = 10
            margin = 10
            spacing = 5
            border_radius = 4

            [behavior]
            wrap_navigation = true
            refresh_interval = 1
            ";

        file.write_all(config_content)?;

        println!("Successfully created whereami.toml in {}", &config_dir);
        Ok(())
    }

    /// if the theme line is uncommented it will default to that, otherwise it will create own
    /// theme
    pub fn get_theme(&self) -> Theme {
        match self.theme.as_deref() {
            Some("GruvboxDark") => Theme::GruvboxDark,
            Some("GruvboxLight") => Theme::GruvboxLight,
            Some("CatppuccinLatte") => Theme::CatppuccinLatte,
            Some("CatppuccinFrappe") => Theme::CatppuccinFrappe,
            Some("CatppuccinMacchiato") => Theme::CatppuccinMacchiato,
            Some("CatppuccinMocha") => Theme::CatppuccinMocha,
            Some("Dracula") => Theme::Dracula,
            Some("Nord") => Theme::Nord,
            Some("SolarizedLight") => Theme::SolarizedLight,
            Some("SolarizedDark") => Theme::SolarizedDark,
            Some("TokyoNight") => Theme::TokyoNight,
            Some("TokyoNightStorm") => Theme::TokyoNightStorm,
            Some("TokyoNightLight") => Theme::TokyoNightLight,
            Some("KanagawaWave") => Theme::KanagawaWave,
            Some("KanagawaDragon") => Theme::KanagawaDragon,
            Some("KanagawaLotus") => Theme::KanagawaLotus,
            Some("Moonfly") => Theme::Moonfly,
            Some("Nightfly") => Theme::Nightfly,
            Some("Oxocarbon") => Theme::Oxocarbon,
            _ => self.to_theme(), // fallback to custom colors
        }
    }
    /// parses Color struct into actual machine readable code
    pub fn to_theme(&self) -> Theme {
        let palette = Palette {
            background: parse_color(&self.colors.background),
            text: parse_color(&self.colors.text),
            primary: parse_color(&self.colors.selected_background),
            success: parse_color(&self.colors.background),
            danger: parse_color(&self.colors.background),
        };

        Theme::custom("user-made palette".to_string(), palette)
    }
}

pub fn parse_color(val: &String) -> Color {
    let hex = val.trim_start_matches("#");
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::from_rgb8(r, g, b)
}
