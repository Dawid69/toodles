use std::{sync::mpsc::Receiver, time::Duration};

use tui::widgets::ListState;

use crate::{
    read_database, remove_selected_task_from_list, MenuItem, SHOW_POPUP, TERMINATION_LOCK,
};

use crate::Event;

use crossterm::event::{self, KeyCode};

pub fn handle_keys(
    rx: &Receiver<Event<event::KeyEvent>>,

    active_menu_item: &mut MenuItem,
    todo_list_state: &mut ListState,
) {
    if let Ok(event) = rx.recv_timeout(Duration::from_millis(500)) {
        if *SHOW_POPUP.lock().unwrap() {
            // Handle keycodes for popups!!

            match active_menu_item {
                MenuItem::Home => todo!(),
                MenuItem::Todo => match event {
                    Event::Input(event) => match event.code {
                        KeyCode::Char('q') => {
                            *SHOW_POPUP.lock().unwrap() = false;
                        }
                        KeyCode::Char('n') => {}
                        _ => {}
                    },
                    Event::Tick => {}
                },
                MenuItem::Done => todo!(),
                MenuItem::All => todo!(),
                MenuItem::Notes => todo!(),
                MenuItem::Quit => todo!(),
            }
        } else {
            // Handle default keys here!!
            match active_menu_item {
                MenuItem::Home => match event {
                    Event::Input(event) => match event.code {
                        KeyCode::Left | KeyCode::Char('h') => active_menu_item.previous(),
                        KeyCode::Right | KeyCode::Char('l') => active_menu_item.next(),
                        KeyCode::Char('q' | 'Q') => {
                            let mut x = TERMINATION_LOCK.lock().unwrap();
                            *x = true;
                            drop(x);
                        }
                        // Menu context sensitive items are shown here!!
                        _ => {}
                    },
                    Event::Tick => {}
                },
                MenuItem::Todo => {
                    match event {
                        Event::Input(event) => match event.code {
                            KeyCode::Left | KeyCode::Char('h') => active_menu_item.previous(),
                            KeyCode::Right | KeyCode::Char('l') => active_menu_item.next(),
                            KeyCode::Char('q' | 'Q') => {
                                let mut x = TERMINATION_LOCK.lock().unwrap();
                                *x = true;
                                drop(x);
                            }
                            // Menu context sensitive items are shown here!!
                            KeyCode::Char('a' | 'A') => {
                                let mut x = SHOW_POPUP.lock().unwrap();
                                *x = true;
                                drop(x);
                            }
                            KeyCode::Char('d' | 'D') => {
                                remove_selected_task_from_list(todo_list_state)
                                    .expect("Cannot remove item!!")
                            }

                            KeyCode::Down | KeyCode::Char('j') => {
                                if let Some(selected) = todo_list_state.selected() {
                                    let amount_of_todos =
                                        read_database().expect("Cannot read DB").len();

                                    if selected >= amount_of_todos - 1 {
                                        todo_list_state.select(Some(0))
                                    } else {
                                        todo_list_state.select(Some(selected + 1));
                                    }
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if let Some(selected) = todo_list_state.selected() {
                                    let amount_of_todos =
                                        read_database().expect("Cannot read DB").len();

                                    if selected > 0 {
                                        todo_list_state.select(Some(selected - 1));
                                    } else {
                                        todo_list_state.select(Some(amount_of_todos - 1));
                                    }
                                }
                            }
                            _ => {}
                        },
                        Event::Tick => {}
                    }
                }
                MenuItem::Done => {
                    match event {
                        Event::Input(event) => match event.code {
                            KeyCode::Left | KeyCode::Char('h' | 'H') => active_menu_item.previous(),
                            KeyCode::Right | KeyCode::Char('l' | 'L') => active_menu_item.next(),
                            KeyCode::Char('q' | 'Q') => {
                                let mut x = TERMINATION_LOCK.lock().unwrap();
                                *x = true;
                                drop(x);
                            }
                            // Menu context sensitive items are shown here!!
                            _ => {}
                        },
                        Event::Tick => {}
                    }
                }
                MenuItem::All => {
                    match event {
                        Event::Input(event) => match event.code {
                            KeyCode::Left | KeyCode::Char('h' | 'H') => active_menu_item.previous(),
                            KeyCode::Right | KeyCode::Char('l' | 'L') => active_menu_item.next(),
                            KeyCode::Char('q' | 'Q') => {
                                let mut x = TERMINATION_LOCK.lock().unwrap();
                                *x = true;
                                drop(x);
                            }
                            // Menu context sensitive items are shown here!!
                            _ => {}
                        },
                        Event::Tick => {}
                    }
                }
                MenuItem::Notes => {
                    match event {
                        Event::Input(event) => match event.code {
                            KeyCode::Left | KeyCode::Char('h' | 'H') => active_menu_item.previous(),
                            KeyCode::Right | KeyCode::Char('l' | 'L') => active_menu_item.next(),
                            KeyCode::Char('q' | 'Q') => {
                                let mut x = TERMINATION_LOCK.lock().unwrap();
                                *x = true;
                                drop(x);
                            }
                            // Menu context sensitive items are shown here!!
                            _ => {}
                        },
                        Event::Tick => {}
                    }
                }
                MenuItem::Quit => unreachable!(),
            }
        }
    }
}
