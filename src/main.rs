mod app;
mod error;
mod types;

use app::App;
use error::{Result, KanaError};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::{Duration, Instant}};
use std::fs::{File, OpenOptions};
use std::path::Path;
use types::{AppMode, UserHistory};

const HISTORY_FILE: &str = "kana_history.json";

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    load_history(&mut app)?;

    app.select_next_kana()?;

    let tick_rate = Duration::from_millis(250);
    let res = run_app(&mut terminal, &mut app, tick_rate);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    save_history(&app)?;

    if let Err(err) = res {
        println!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    tick_rate: Duration,
) -> Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| app.render(f))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => {
                        if app.state.mode == AppMode::Ready {
                            app.handle_input(c)
                        }
                    },
                    KeyCode::Enter => {
                        app.handle_enter()?;
                    },
                    KeyCode::Backspace => {
                        if app.state.mode == AppMode::Ready {
                            app.state.input_buffer.pop();
                        }
                    },
                    KeyCode::Esc => {
                        app.should_quit = true;
                    },
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn load_history(app: &mut App) -> Result<()> {
    if Path::new(HISTORY_FILE).exists() {
        let file = File::open(HISTORY_FILE)?;
        app.state.history = serde_json::from_reader(file)?;
    }
    Ok(())
}

fn save_history(app: &App) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(HISTORY_FILE)?;
    
    serde_json::to_writer_pretty(file, &app.state.history)?;
    Ok(())
}