// terminal.rs
use std::{
    io::{self, Stdout},
    time::Duration,
};

use bevy::app::AppExit;
use bevy::prelude::*;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

#[derive(Resource)]
pub struct TuiTerminal(pub Terminal<CrosstermBackend<Stdout>>);

impl Drop for TuiTerminal {
    fn drop(&mut self) {
        // se ejecuta al eliminar el recurso (ej. al cerrar la app)
        let _ = disable_raw_mode();
        let _ = execute!(self.0.backend_mut(), LeaveAlternateScreen);
        let _ = self.0.show_cursor();
    }
}

pub fn setup_terminal(mut commands: Commands) {
    enable_raw_mode().expect("failed to enable raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).expect("failed to enter alternate screen");
    let terminal =
        Terminal::new(CrosstermBackend::new(stdout)).expect("failed to create terminal");
    commands.insert_resource(TuiTerminal(terminal));
}

pub struct TuiPlugin;

impl Plugin for TuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_terminal)
            .add_systems(Update, handle_quit_input);
    }
}

fn handle_quit_input(mut exit: MessageWriter<AppExit>) {
    while event::poll(Duration::from_millis(0)).unwrap_or(false) {
        let Ok(Event::Key(key)) = event::read() else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }

        let quit = matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL));
        if quit {
            exit.write(AppExit::Success);
        }
    }
}
