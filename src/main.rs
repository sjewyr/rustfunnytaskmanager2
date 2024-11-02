use std::process;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use funnytaskmanager2::run;

fn main() {
    funnytaskmanager2::migrations()
        .inspect_err(|err| {
            eprintln!("Error with migrations: {err}");
            process::exit(1);
        })
        .ok();
    
    run();
    
    
    ratatui::restore();
}
