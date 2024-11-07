use chrono::Utc;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use rusqlite::{self, Connection};
use tui_input::{backend::crossterm::EventHandler, Input};

enum Opened {
    View,
    Insert,
}

struct MyState {
    list_state: ListState,
    items: Vec<Task>,
    state: Opened,
    input: Input,
}
impl<'a> Into<ListItem<'a>> for &Task {
    fn into(self) -> ListItem<'a> {
        ListItem::new(format!("{}: {}", self.name, self.date))
    }
}
struct Task {
    id: u64,
    name: String,
    date: chrono::DateTime<Utc>,
}
pub fn run() {
    let conn = Connection::open("./data.db3").expect("failed to connect to db");
    let mut terminal = ratatui::init();
    let items = fetch_tasks(&conn)
        .inspect_err(|err| eprintln!("{err}"))
        .unwrap_or_default();

    let mut my_state = MyState {
        list_state: ListState::default(),
        items,
        state: Opened::View,
        input: Input::default(),
    };
    loop {
        match my_state.state {
            Opened::View => {
                terminal
                    .draw(|frame| draw(frame, &mut my_state))
                    .expect("Failed to draw");
            }
            Opened::Insert => {
                terminal
                    .draw(|frame| draw_input(frame, &mut my_state))
                    .expect("Failed to draw");
            }
        }

        if let Event::Key(key) = event::read().expect("failed to read event") {
            if key.kind == KeyEventKind::Press {
                match my_state.state {
                    Opened::View => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up => my_state.list_state.select_previous(),
                        KeyCode::Down => my_state.list_state.select_next(),
                        KeyCode::Char('i') => my_state.state = Opened::Insert,
                        KeyCode::Backspace => {
                            if let Some(index) = my_state.list_state.selected() {
                                del_task(my_state.items[index].id, &conn)
                                    .inspect_err(|err| eprintln!("{err}"))
                                    .ok();
                                my_state.items = fetch_tasks(&conn)
                                    .inspect_err(|err| eprintln!("{err}"))
                                    .unwrap_or_default();
                            }
                        }
                        _ => continue,
                    },
                    Opened::Insert => match key.code {
                        KeyCode::Esc => {
                            my_state.state = Opened::View;
                            my_state.input.reset();
                        }
                        KeyCode::Enter => {
                            my_state.state = Opened::View;
                            insert_new_task(my_state.input.value().to_string(), &conn)
                                .inspect_err(|err| eprintln!("{err}"))
                                .ok();
                            my_state.items = fetch_tasks(&conn)
                                .inspect_err(|err| eprintln!("{err}"))
                                .unwrap_or_default();
                            my_state.input.reset();
                        }
                        _ => {
                            my_state.input.handle_event(&Event::Key(key));
                        }
                    },
                }
            }
        }
    }
}
fn insert_new_task(content: String, conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "Insert INTO task(name, date) VALUES(?1, (SELECT datetime()))",
        (&content,),
    )?;
    Ok(())
}

fn del_task(id: u64, conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM task WHERE id == ?1", (id,))?;
    Ok(())
}

fn fetch_tasks(conn: &Connection) -> Result<Vec<Task>, rusqlite::Error> {
    let mut items = Vec::new();
    conn.prepare("Select id, name, date from task")?
        .query_map((), |item| {
            Ok(Task {
                id: item.get(0)?,
                name: item.get(1)?,
                date: item.get(2)?,
            })
        })?
        .try_for_each(|row| -> Result<(), rusqlite::Error> {
            items.push(row?);
            Ok(())
        })?;

    Ok(items)
}

pub fn migrations() -> Result<(), String> {
    let conn =
        Connection::open("./data.db3").map_err(|err| format!("Err creating/opening db: {err}"))?;

    let migration_vec = vec![
        "CREATE TABLE IF NOT EXISTS task (
            id INTEGER PRIMARY KEY,
            name VARCHAR(255),
            date datetime
    )",
    ];
    let i: rusqlite::Result<()> =
        migration_vec
            .iter()
            .try_for_each(|migration| -> rusqlite::Result<()> {
                conn.execute(migration, ())?;
                Ok(())
            });
    i.map_err(|err| format!("Err with migrations: {err}"))?;
    Ok(())
}

fn draw(frame: &mut Frame, state: &mut MyState) {
    let list = List::new(&state.items)
        .block(Block::bordered().title("List"))
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true);
    let layout = Layout::vertical([Constraint::Min(20), Constraint::Length(20)]);
    let rect = layout.split(frame.area());
    let text = Text::raw("Q: Quit; UP: select prev; DOWN: select next; I: New; BACKSPACE: Delete");
    frame.render_stateful_widget(list, rect[0], &mut state.list_state);
    frame.render_widget(text, rect[1]);
}

fn draw_input(frame: &mut Frame, state: &mut MyState) {
    let inp = &state.input;
    let scroll = inp.visual_scroll(100 as usize);
    let input = Paragraph::new(inp.value())
        .style(Style::default().fg(Color::Yellow))
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("New task"));
    let layout = Layout::vertical([Constraint::Min(20), Constraint::Length(20)]);
    let rect = layout.split(frame.area());
    let text = Text::raw("ESC: Back to main menu; ENTER: Submit");
    frame.render_widget(input, rect[0]);
    frame.render_widget(text, rect[1]);
}
