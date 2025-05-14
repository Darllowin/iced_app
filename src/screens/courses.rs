use iced::{widget::{Text, Column, Container}, Alignment};
use iced_aw::{card, style};
use rusqlite::Connection;
use crate::app::{App, Message};
use crate::db::{self};

pub fn courses_screen(_app: &App) -> Container<Message> {
    let conn = Connection::open("db_platform").unwrap();
    let courses = db::get_courses(&conn).unwrap_or_default();

    let mut column = Column::new().spacing(20).padding(20);

    for course in courses {
        column = column.push(
            Column::new()
                .push(
                    card(
                        Text::new(format!("{}", course.title)).size(24),
                        Column::new().push(Text::new(format!("{}", course.description)).size(16))
                    )
                        .style(style::card::dark)
                )
        );
    }
    Container::new(column)
        .align_y(Alignment::Start)
}
