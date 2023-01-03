const DB_PATH: &str = "./data/db.json";

mod renders;
mod user_input;

use std::{
    fs, io, process,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use chrono::prelude::Local;
use chrono::DateTime;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use renders::{
    centered_rect, render_static_home, render_static_test_page, render_todo_page, split_main_window,
};
use serde::{Deserialize, Serialize};

use crossterm::event::Event as CrossEvent;
use strum_macros::Display;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, ListState, Paragraph, Tabs},
    Terminal,
};

use strum::{EnumIter, IntoEnumIterator};

static SHOW_POPUP: Mutex<bool> = Mutex::new(false);
static TERMINATION_LOCK: Mutex<bool> = Mutex::new(false);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // arc that will handle breaking and cleaning up maybe???
    // let termination_lock = Arc::new(Mutex::new(false));

    // Arc that determines whether there will be a popup showing....
    // let show_popup = Arc::new(Mutex::new(false));

    let (tx, rx): (
        Sender<Event<event::KeyEvent>>,
        Receiver<Event<event::KeyEvent>>,
    ) = channel();
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();

        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_millis(0));

            if event::poll(timeout).expect("poll works") {
                if let CrossEvent::Key(key) = event::read().expect("Cannot read events") {
                    tx.send(Event::Input(key)).expect("Cant send events");
                }
            }

            // if timeout has elapsed, send Tick Event on channel and reset last tick
            if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });

    // Create menu stuff
    let menu_titles = create_menu_vector();

    let mut active_menu_item = MenuItem::Home;

    let mut stdout_value = io::stdout();

    execute!(stdout_value, EnterAlternateScreen, EnableMouseCapture)
        .expect("I cannot do that steven");
    enable_raw_mode().expect("Cannot enable raw mode");

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout_value)).unwrap();

    if let Err(e) = terminal.clear() {
        println!("Error clearing terminal: {}", e);
        process::exit(5);
    }

    // init list states here for stateful widgets
    let mut todo_list_state = ListState::default();
    todo_list_state.select(Some(0));

    // implement the render loop

    '_renderLoop: loop {
        terminal
            .draw(|frame| {
                let sub_win = split_main_window(frame);

                let static_widget = Paragraph::new("STATIC FOOTER THAT IS HERE!!\n\nYEEEE")
                    .style(Style::default().fg(Color::Magenta))
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default().fg(Color::DarkGray))
                            .title("DEF WIDGET")
                            .border_type(BorderType::Thick),
                    );

                // Create menu items from hardcoded vector, split each one off at the first letter and
                // give it a different highlighting to show what shortcut corresponds to it... Could be
                // implemented better in some way...
                let menu: Vec<Spans> = menu_titles
                    .iter()
                    .map(|t| {
                        let (first, rest) = t.split_at(1);

                        Spans::from(vec![
                            Span::styled(
                                first,
                                Style::default()
                                    .fg(Color::Blue)
                                    .add_modifier(Modifier::RAPID_BLINK),
                            ),
                            Span::styled(rest, Style::default().fg(Color::Cyan)),
                        ])
                    })
                    .collect();

                let tabs = Tabs::new(menu)
                    .select(active_menu_item.into())
                    .block(Block::default().title("Menu").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED),
                    )
                    .divider(Span::raw("||"));

                frame.render_widget(static_widget, sub_win[2]);

                frame.render_widget(tabs, sub_win[0]);

                // Handle input here

                /////////////////////////////////////////////////////////////////////////////////
                user_input::handle_keys(&rx, &mut active_menu_item, &mut todo_list_state);
                //////////////////////////////////////////////////////////////////////////////

                // Decide what to render on the main screen here...

                match active_menu_item {
                    MenuItem::Home => frame.render_widget(render_static_home(), sub_win[1]),
                    MenuItem::Todo => {
                        // separate subwindow into smaller subwindows..
                        let todo_subwin = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(
                                [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                            )
                            .split(sub_win[1]);

                        // split bigger window into vertical arrangement to have a details area...

                        let todo_detail = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints(
                                [Constraint::Percentage(10), Constraint::Percentage(90)].as_ref(),
                            )
                            .split(todo_subwin[1]);

                        let (left, (right_top, right_bottom)) = render_todo_page(&todo_list_state);

                        frame.render_stateful_widget(left, todo_subwin[0], &mut todo_list_state);
                        frame.render_widget(right_top, todo_detail[0]);
                        frame.render_widget(right_bottom, todo_detail[1]);
                    }
                    MenuItem::Done => frame.render_widget(render_static_test_page(), sub_win[1]),
                    MenuItem::All => frame.render_widget(render_static_test_page(), sub_win[1]),
                    MenuItem::Notes => frame.render_widget(render_static_test_page(), sub_win[1]),
                    MenuItem::Quit => unreachable!(),
                }

                // Decide whether popups need to be drawn over the current screen...

                let x = SHOW_POPUP.lock().unwrap();
                if *x {
                    let size = frame.size();

                    let block = Block::default()
                        .title(" Add New Todo ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Double);
                    let area = centered_rect(90, 80, size);

                    frame.render_widget(Clear, area);
                    frame.render_widget(block, area);
                }
            })
            .expect("BIIIG ERROR"); // END OF DRAW

        let x = TERMINATION_LOCK.lock().unwrap();
        if *x {
            // TODO add saving and cleaning up data here?

            execute!(
                terminal.backend_mut(),
                DisableMouseCapture,
                LeaveAlternateScreen
            )?;

            disable_raw_mode().expect("Cannot disable raw mode");
            terminal.show_cursor().expect("Cannot show cursor?? ");
            terminal.clear().expect("Cannot clear terminal");
            return Ok(()); // Exit program
        }
        drop(x);
    }
}

fn add_new_test_task_to_list() -> Result<(), LocalError> {
    // todo read db here and find the last entries ID....

    let x = Task {
        id: rand::thread_rng().gen_range(0, 100),
        name: "NEW DEFAULT NAME".to_string(),
        description: "NEW REALLY LONG HOPELLY NOT TRUNCATED DESCRIPTION THAT WILL HAVE TO BE EITHER TWEAKED OR FIXED OR SOMETHING LIKE THAT IN THE END..............   hOPEFULLY!! MAYBE WITH A FEW NEWLINES?   \n lEYTS SEE IF THIS RENDERS.... \n OR THIS MAYBE \n MAYBE HERE... ".to_string(),
        complete: false,
        priority:rand::thread_rng().gen_range(0, 10),
        created: Local::now(),
    };

    if let Ok(mut data) = read_database() {
        data.push(x);

        write_database(&data);

        Ok(())
        // Ok(data)
    } else {
        Err(LocalError::CannotReadFromFile)
    }
}

fn remove_selected_task_from_list(list_state: &mut ListState) -> Result<(), LocalError> {
    if let Some(selected_item) = list_state.selected() {
        let mut list = read_database().expect("Cannot read DB");

        if list.is_empty() {
            return Ok(());
        }

        list.remove(selected_item);

        write_database(&list);

        if selected_item == 0 {
            list_state.select(Some(0));
        } else {
            list_state.select(Some(selected_item - 1));
        }
    }

    Ok(())
}

fn read_database() -> Result<Vec<Task>, LocalError> {
    let raw_content = fs::read_to_string(DB_PATH).expect("Cannot read DB");

    let parsed_data: Vec<Task> = serde_json::from_str(&raw_content).expect("Cannot parse data!!");

    Ok(parsed_data)
}

fn write_database(x: &Vec<Task>) {
    fs::write(DB_PATH, &serde_json::to_vec(x).unwrap()).expect("Cannot Write DB");
}
/// Create a vector of all menu titles
fn create_menu_vector() -> Vec<String> {
    let menu_title: Vec<_> = MenuItem::iter().collect::<Vec<_>>();

    println!("menu_titles : {:?}", menu_title);

    let mut vec: Vec<String> = Vec::new();

    for item in &menu_title {
        let x = item.to_string();
        vec.push(x.clone());
    }

    vec
}
#[derive(Serialize, Deserialize, Clone)]
struct Task {
    id: usize,
    name: String,
    description: String,
    complete: bool,
    priority: usize,
    created: DateTime<Local>,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Empty List".into(),
            description: "".into(),
            complete: false,
            priority: 0,
            created: DateTime::default(),
        }
    }
}

pub enum Event<I> {
    Input(I),
    Tick,
}

enum _NoteType<'a> {
    Short(&'a str),
    Long(&'a str),
}

#[derive(Clone, Copy, Debug, EnumIter, Display)]
pub enum MenuItem {
    Home,
    Todo,
    Done,
    All,
    Notes,
    Quit,
}

impl MenuItem {
    fn next(&mut self) {
        match self {
            MenuItem::Home => *self = MenuItem::Todo,
            MenuItem::Todo => *self = MenuItem::Done,
            MenuItem::Done => *self = MenuItem::All,
            MenuItem::All => *self = MenuItem::Notes,
            MenuItem::Notes => *self = MenuItem::Home,
            MenuItem::Quit => unreachable!(),
        }
    }

    fn previous(&mut self) {
        match self {
            MenuItem::Home => *self = MenuItem::Notes,
            MenuItem::Todo => *self = MenuItem::Home,
            MenuItem::Done => *self = MenuItem::Todo,
            MenuItem::All => *self = MenuItem::Done,
            MenuItem::Notes => *self = MenuItem::All,
            MenuItem::Quit => unreachable!(),
        }
    }
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Todo => 1,
            MenuItem::Done => 2,
            MenuItem::All => 3,
            MenuItem::Notes => 4,
            MenuItem::Quit => 5,
        }
    }
}

// TODO use this error stuff for some reason....
#[derive(Debug)]
pub enum LocalError {
    Generic,
    CannotReadFromFile,
}
