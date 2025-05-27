use std::fs;
use std::path::Path;
use chrono::Local;
use iced::Theme;
use crate::app::state::{Config, CONFIG_FILE, PATH_TO_DB};

pub fn theme_from_str(name: &str) -> Option<Theme> {
    Theme::ALL
        .iter()
        .find(|t| theme_to_str(t).eq_ignore_ascii_case(name))
        .cloned()
}

pub fn save_config(theme: &Theme, interval: Option<&str>, folder: Option<String>, max_count: Option<usize>) -> std::io::Result<()> {
    let config = Config {
        theme_name: theme_to_str(theme).to_string(),
        backup_interval: interval.map(|s| s.to_string()),
        backup_folder: folder,
        max_backup_count: max_count,
    };
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(CONFIG_FILE, json)?;
    Ok(())
}
pub fn backup_database_now() -> std::io::Result<()> {
    let backup_dir = Path::new("backup");

    // Создаём папку, если она не существует
    if !backup_dir.exists() {
        fs::create_dir_all(backup_dir)?;
    }

    // Формируем имя файла резервной копии с меткой времени
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let backup_filename = format!("backup_{}.db", timestamp);
    let backup_path = backup_dir.join(backup_filename);

    // Копируем БД в указанное место
    fs::copy(PATH_TO_DB, backup_path)?;

    Ok(())
}
pub fn load_config() -> Option<Config> {
    let contents = fs::read_to_string(CONFIG_FILE).ok()?;
    serde_json::from_str(&contents).ok()
}
pub fn theme_to_str(theme: &Theme) -> &'static str {
    match theme {
        Theme::Light => "Light",
        Theme::Dark => "Dark",
        Theme::Dracula => "Dracula",
        Theme::Nord => "Nord",
        Theme::SolarizedLight => "SolarizedLight",
        Theme::SolarizedDark => "SolarizedDark",
        Theme::GruvboxLight => "GruvboxLight",
        Theme::GruvboxDark => "GruvboxDark",
        Theme::CatppuccinLatte => "CatppuccinLatte",
        Theme::CatppuccinFrappe => "CatppuccinFrappe",
        Theme::CatppuccinMacchiato => "CatppuccinMacchiato",
        Theme::CatppuccinMocha => "CatppuccinMocha",
        Theme::TokyoNight => "TokyoNight",
        Theme::TokyoNightStorm => "TokyoNightStorm",
        Theme::TokyoNightLight => "TokyoNightLight",
        Theme::KanagawaWave => "KanagawaWave",
        Theme::KanagawaDragon => "KanagawaDragon",
        Theme::KanagawaLotus => "KanagawaLotus",
        Theme::Moonfly => "Moonfly",
        Theme::Nightfly => "Nightfly",
        Theme::Oxocarbon => "Oxocarbon",
        Theme::Ferra => "Ferra",
        _ => "Unknown",
    }
}