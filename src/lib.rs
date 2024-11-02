use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{layout::{Constraint, Layout}, style::{Style, Stylize}, text::Text, widgets::{Block, List, ListState}, Frame};
use rusqlite::{self, Connection};

struct MyState{
    list_state: ListState

}
pub fn run() {
    let mut terminal = ratatui::init();
    let mut my_state = MyState{list_state: ListState::default()};
    loop {
        terminal.draw(|frame| draw(frame, &mut my_state)).expect("Failed to draw");

        if let Event::Key(key) = event::read().expect("failed to read event") {
            if key.kind == KeyEventKind::Press {
                if key.code == KeyCode::Char('q') {
                    break;
                }
                if key.code == KeyCode::Up{
                    my_state.list_state.select_previous();
                }
                if key.code == KeyCode::Down{
                    my_state.list_state.select_next();
                }
            }
        }
    }
}
pub fn migrations() -> Result<(), String> {
    let conn = Connection::open("./data.db3").map_err(|err| format!("Err creating/opening db: {err}"))?;
    
    let migration_vec = vec![
        "CREATE TABLE IF NOT EXISTS task (
            id INTEGER PRIMARY KEY,
            name VARCHAR(255),
            date datetime
    )",
        
    ];
    let i: rusqlite::Result<()> = migration_vec
        .iter()
        .try_for_each(|migration| -> rusqlite::Result<()> {
            conn.execute(migration, ())?;
            Ok(())
        }
    );
    i.map_err(|err| format!("Err with migrations: {err}"))?;
    Ok(())
}

pub fn draw(frame: &mut Frame, state: &mut MyState){
    let items = ["Item 1", "Item 2", "Item 3"];
    let list = List::new(items)
        .block(Block::bordered().title("List"))
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true);
    let layout = Layout::vertical([Constraint::Length(20), Constraint::Min(20), Constraint::Percentage(75)]);
    let rect = layout.split(frame.area());
    frame.render_stateful_widget(list, rect[0], &mut state.list_state);
}

