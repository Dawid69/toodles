use std::io::Stdout;

use rand::Rng;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Wrap,
    },
    Frame,
};

use crate::{read_database, Task};

pub fn split_main_window(
    frame: &mut Frame<CrosstermBackend<Stdout>>,
) -> std::vec::Vec<tui::layout::Rect> {
    let size = frame.size();
    // A chunk is a section of window as defined in the let fn
    let sub_win = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3), // Top
                Constraint::Min(2),    // Middle Min indicates that it will be dynamic
                Constraint::Length(3), // Bottom
            ]
            .as_ref(),
        )
        .split(size);

    sub_win
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn render_todo_page<'a>(todo_list_state: &ListState) -> (List<'a>, (Table<'a>, Paragraph<'a>)) {
    // create block
    let todo_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Todo")
        .border_type(BorderType::Plain);

    // fetch items from DB
    let todo_list = read_database().expect("Cannot read database!!");

    // create list item that will have name of the individual items...
    let items: Vec<_> = todo_list
        .iter()
        .map(|entry| {
            ListItem::new(Spans::from(vec![Span::styled(
                entry.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    // set the the currently selected item

    let default_task = Task::default();

    let selected_todo = todo_list
        .get(
            todo_list_state
                .selected()
                .expect("Should not error out as there is always something selected... "),
        )
        .unwrap_or(&default_task);

    // create the list of todo entries

    let list = List::new(items).block(todo_block).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    // render a table for the currently selected item
    let todo_detail = Table::new(vec![
        // Only has one row.... Might have to add another...
        Row::new(vec![
            // Cells in row
            Cell::from(Span::raw(selected_todo.id.to_string())),
            Cell::from(Span::raw(selected_todo.name.clone())),
            Cell::from(Span::raw(selected_todo.complete.to_string())),
            Cell::from(Span::raw(selected_todo.priority.to_string())),
            Cell::from(Span::raw(selected_todo.created.to_string())),
        ]),
    ])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Complete",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Priority",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Created",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Detail")
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(30),
    ]);

    let todo_desc = Paragraph::new(selected_todo.description.clone()).wrap(Wrap { trim: true });

    (list, (todo_detail, todo_desc))
    // return both widgets to be rendered....
}

pub fn render_static_test_page<'a>() -> Paragraph<'a> {
    let mut rng = rand::thread_rng();

    let r = rng.gen_range(0, 255);
    let g = rng.gen_range(0, 255);
    let b = rng.gen_range(0, 255);

    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "RANDOM!!",
            Style::default().fg(Color::Rgb(r, g, b)),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "toodles",
            Style::default().fg(Color::LightBlue),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );

    home
}

pub fn render_static_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "toodles",
            Style::default().fg(Color::LightBlue),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );

    home
}
