use std::{
    io::{self, Stdout},
    time::Duration,
};

use bevy::app::AppExit;
use bevy::prelude::*;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{ui::Camera, world::Board};

use super::render;

#[derive(Resource)]
pub struct TuiTerminal(pub Terminal<CrosstermBackend<Stdout>>);

impl Drop for TuiTerminal {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.0.backend_mut(),
            DisableMouseCapture,
            LeaveAlternateScreen
        );
        let _ = self.0.show_cursor();
    }
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen);
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
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .expect("failed to enter alternate screen");
    let terminal = Terminal::new(CrosstermBackend::new(stdout)).expect("failed to create terminal");
    commands.insert_resource(TuiTerminal(terminal));
}

pub struct TuiPlugin;

impl Plugin for TuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_terminal)
            .add_systems(Update, (handle_input, render));
    }
}

fn handle_input(mut exit: MessageWriter<AppExit>, mut camera: ResMut<Camera>, board: Res<Board>) {
    while event::poll(Duration::from_millis(0)).unwrap_or(false) {
        // 1. Leemos el evento general sin filtrar todavía
        let Ok(current_event) = event::read() else {
            continue;
        };

        // 2. Procesamos según el tipo de evento (Teclado o Mouse)
        match current_event {
            // === MANEJO DE TECLADO ===
            Event::Key(key) => {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                let quit = matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL));

                if quit {
                    exit.write(AppExit::Success);
                    continue;
                }

                let (dx, dy) = match key.code {
                    KeyCode::Up => (0, -1),
                    KeyCode::Down => (0, 1),
                    KeyCode::Left => (-1, 0),
                    KeyCode::Right => (1, 0),
                    _ => continue,
                };

                let (view_w, view_h) = (camera.last_view_w, camera.last_view_h);
                camera.move_by(dx, dy, &board, view_w, view_h);
            }

            // === MANEJO DE MOUSE (DESPLAZAMIENTO DEL MAPA) ===
            Event::Mouse(MouseEvent {
                kind, column, row, ..
            }) => {
                match kind {
                    MouseEventKind::Down(MouseButton::Middle) => {
                        camera.is_dragging = true;
                        camera.last_mouse_pos = Some((column, row));
                    }
                    MouseEventKind::Drag(MouseButton::Middle) | MouseEventKind::Moved => {
                        if camera.is_dragging {
                            if let Some((last_x, last_y)) = camera.last_mouse_pos {
                                let delta_x = column as i32 - last_x as i32;
                                let delta_y = row as i32 - last_y as i32;
                                let (view_w, view_h) = (camera.last_view_w, camera.last_view_h);
                                camera.move_by(
                                    -delta_x as isize,
                                    -delta_y as isize,
                                    &board,
                                    view_w,
                                    view_h,
                                );
                                camera.last_mouse_pos = Some((column, row));
                            }
                        }
                    }
                    MouseEventKind::Up(MouseButton::Middle) => {
                        camera.is_dragging = false;
                        camera.last_mouse_pos = None;
                    }
                    _ => {}
                }
            }

            _ => {} // Ignoramos Resize u otros eventos
        }
    }
}
