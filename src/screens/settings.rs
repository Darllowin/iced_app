use crate::app::{App, Message};
use iced::widget::{button, pick_list, row};
use iced::{widget::{column, text, Container}, Length, Theme};
use crate::app::state::{BackupInterval, BACKUP_INTERVALS};
use crate::config::theme_to_str;

pub fn settings_screen(app: &App) -> Container<Message> {
    let current_theme_name = theme_to_str(&app.theme);
    let theme_names: Vec<&'static str> = Theme::ALL.iter().map(theme_to_str).collect();
    
    let max_backup_options = vec![3, 5, 10, 20];

    let content = column![
        row![
            text("Настройки").size(30),
        ].padding(10),
        row![
            column![
                text("Тема приложения").size(26),
                pick_list(theme_names, Some(current_theme_name), Message::ThemeSelected)
                    .placeholder("Выберите тему"),
                text("Период резервного копирования").size(26),
                pick_list(
                    BACKUP_INTERVALS.to_vec(),
                    app.backup_interval.clone(),
                    |value: BackupInterval| Message::BackupIntervalSelected(Some(value)),
                ).placeholder("Период резервного копирования"),
                row![
                    text("Папка для бэкапов: "),
                    button("Выбрать").on_press(Message::SelectBackupFolder),
                    text(app.backup_folder.clone().unwrap_or("Не выбрана".to_string())),
                ]
                 .spacing(10),
                  pick_list(
                     max_backup_options,
                     app.max_backup_count,
                     |value| Message::MaxBackupCountSelected(Some(value)),
                  ).placeholder("Максимум резервных копий"),
                button("Сделать резервную копию сейчас").on_press(Message::BackupNowPressed),
            ].spacing(10).padding(10),
        ],
    ].spacing(15);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)
}

