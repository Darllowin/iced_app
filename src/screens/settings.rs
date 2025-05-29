// settings_screen.rs
use crate::app::{App, Message};
use iced::widget::{button, container, pick_list, row, text_input, vertical_space, Row};
use iced::{
    widget::{column, text, Container, tooltip, Space},
    Length, Theme, Element, Border,
    theme::palette::{Extended, Pair}, // Импорт для палитры
};
use crate::app::state::{BackupInterval, BACKUP_INTERVALS};
use crate::config::theme_to_str;
use iced_anim::Animation;
use iced_font_awesome::fa_icon_solid;
use crate::app::update::icon_button_content;
// Добавьте этот импорт

pub fn settings_screen(app: &App) -> Container<Message> {
    let current_theme_name = theme_to_str(app.theme.target()); // Используйте app.theme.target() для начального выбора
    let theme_names: Vec<&'static str> = Theme::ALL.iter().map(theme_to_str).collect();

    let max_backup_options = vec![3, 5, 10, 20];

    let content = column![
        row![
            text("Настройки").size(30),
        ].padding(10),
        row![
            column![
                text("Тема приложения").size(26),
                pick_list(theme_names, Some(current_theme_name), |name| {
                    Message::ThemeSelected(name)
                })
                .placeholder("Выберите тему"),
                palette_grid(app.theme.value().extended_palette()),
                vertical_space(),
                
            ].spacing(10).padding(10),
            column![
                text("Период резервного копирования").size(26),
                pick_list(
                    BACKUP_INTERVALS.to_vec(),
                    app.backup_interval.clone(),
                    |value: BackupInterval| Message::BackupIntervalSelected(Some(value)),
                ).placeholder("Период резервного копирования"),
                row![
                    text("Папка для бэкапов: ").center(),
                    button("Выбрать").on_press(Message::SelectBackupFolder),
                    text_input("",&*app.backup_folder.clone().unwrap_or("Не выбрана".to_string())).on_input(Message::Er),
                    //vertical_space()
                ].spacing(10),
                text("Последний бэкап:").size(26),
                text(app.last_backup_time.clone().unwrap_or("Не найден".to_string())),
                button(icon_button_content(
                    fa_icon_solid("folder-closed").style(move |_| text::base(&app.theme.target())),
                    "Открыть папку с бэкапами"
                )).on_press(Message::OpenBackupFolder),
                text("Максимум резервных копий").size(26),
                pick_list(
                    max_backup_options,
                    app.max_backup_count,
                    |value| Message::MaxBackupCountSelected(Some(value)),
                ).placeholder("Максимум резервных копий"),
                button(icon_button_content(
                        fa_icon_solid("database").style(move |_| text::base(&app.theme.target())),
                        "Сделать резервную копию сейчас"
                    )).on_press(Message::BackupNowPressed),
            ].spacing(10).padding(10)
        ],
    ].spacing(20);
    
    Container::new(
        Animation::new(
            &app.theme, 
            Container::new(content) 
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(40) 
        ).on_update(Message::ChangeTheme) 
    )
        .width(Length::Fill)
        .height(Length::Fill)
}

// Функции для отображения цветовой палитры
fn palette_grid<'a>(palette: &Extended) -> Element<'a, Message> {
    // Различные оттенки палитры
    let shades = [
        (
            "Primary",
            palette.primary.strong,
            palette.primary.base,
            palette.primary.weak,
        ),
        (
            "Secondary",
            palette.secondary.strong,
            palette.secondary.base,
            palette.secondary.weak,
        ),
        (
            "Success",
            palette.success.strong,
            palette.success.base,
            palette.success.weak,
        ),
        (
            "Danger",
            palette.danger.strong,
            palette.danger.base,
            palette.danger.weak,
        ),
        (
            "Background",
            palette.background.strong,
            palette.background.base,
            palette.background.weak,
        ),
    ];

    Container::new(row![
        // Используйте `Row::with_children` для итерации и построения строки
        Row::with_children(shades.into_iter().map(
            |(name, strong, base, weak)| {
                column![
                    pair_square(format!("{name} weak"), weak),
                    pair_square(format!("{name} base"), base),
                    pair_square(format!("{name} strong"), strong),
                ]
                .into()
            },
        ))
    ])
        .padding(1.0)
        .style(|theme: &Theme| container::Style {
            border: Border {
                color: theme.palette().text,
                width: 1.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn pair_square<'a>(name: String, pair: Pair) -> Element<'a, Message> {
    tooltip(
        Container::new(Space::new(Length::Shrink, Length::Shrink))
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .style(move |_| container::Style {
                background: Some(pair.color.into()),
                ..Default::default()
            }),
        Container::new(text(name).style(|theme: &Theme| text::Style {
            color: Some(theme.palette().text),
        }))
            .style(|theme: &Theme| container::Style {
                background: Some(theme.extended_palette().background.weak.color.into()),
                border: Border {
                    color: theme.extended_palette().background.base.color,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .padding(8),
        tooltip::Position::Top,
    )
        .into()
}