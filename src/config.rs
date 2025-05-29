use std::fs;
use std::path::Path;
use std::time::SystemTime;
use chrono::{DateTime, Local};
use iced::Theme;
use crate::app::state::{BackupInterval, Config, CONFIG_FILE, PATH_TO_DB};

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
pub fn start_backup_scheduler(
    interval: Option<BackupInterval>,
    folder: Option<String>,
    max_copies: Option<usize>,
) {
    if let Some(interval) = interval {
        if let Some(duration) = interval.duration() {
            let folder = folder.unwrap_or_else(|| "backup".to_string());

            std::thread::spawn(move || {
                loop {
                    if let Err(e) = perform_backup(&folder, max_copies) {
                        eprintln!("Ошибка резервного копирования: {}", e);
                    }
                    std::thread::sleep(duration);
                }
            });
        } else {
            println!("Автоматическое резервное копирование отключено.");
        }
    }
}
pub fn perform_backup(backup_dir: &str, max_copies: Option<usize>) -> std::io::Result<()> {
    let backup_path = Path::new(backup_dir);
    if !backup_path.exists() {
        fs::create_dir_all(backup_path)?;
    }

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let backup_filename = format!("backup_{}.db", timestamp);
    let backup_file_path = backup_path.join(backup_filename);

    fs::copy(PATH_TO_DB, &backup_file_path)?;

    // Очистка старых копий
    if let Some(max) = max_copies {
        let mut entries: Vec<_> = fs::read_dir(backup_path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .filter(|e| e.file_name().to_string_lossy().starts_with("backup_"))
            .collect();

        entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH));

        while entries.len() > max {
            if let Some(entry) = entries.first() {
                let path = entry.path();
                let _ = fs::remove_file(path);
                entries.remove(0);
            }
        }
    }

    Ok(())
}
pub fn get_last_backup_time(backup_dir: &str) -> Option<String> {
    let path = Path::new(backup_dir);
    let entries = fs::read_dir(path).ok()?;

    let mut latest: Option<SystemTime> = None;

    for entry in entries.filter_map(Result::ok) {
        let meta = entry.metadata().ok()?;
        if meta.is_file() {
            let modified = meta.modified().ok()?;
            if latest.is_none() || Some(modified) > latest {
                latest = Some(modified);
            }
        }
    }

    latest.map(|time| {
        let datetime: DateTime<Local> = time.into();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    })
}
pub fn backup_database_now_with_config(
    folder: Option<String>,
    max_copies: Option<usize>
) -> std::io::Result<()> {
    let backup_folder = folder.unwrap_or_else(|| "backup".to_string());
    perform_backup(&backup_folder, max_copies)
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