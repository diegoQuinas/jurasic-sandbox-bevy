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

use super::render;

#[derive(Resource)]
pub struct TuiTerminal(pub Terminal<CrosstermBackend<Stdout>>);

impl Drop for TuiTerminal {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.0.backend_mut(), LeaveAlternateScreen);
        let _ = self.0.show_cursor();
    }
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
}

pub fn setup_terminal(mut commands: Commands) {
    // Panic hook must restore the TTY; otherwise a crash leaves raw mode on.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

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
            .add_systems(Update, (handle_quit_input, render));
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
