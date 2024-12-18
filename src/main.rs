mod app;
mod error;
mod kana;
mod types;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use error::{KanaError, Result};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, fs::{File, OpenOptions}};
use std::path::Path;
use std::{
    io,
    time::{Duration, Instant},
};
use types::{AppMode, PracticeMode, UserHistory};

use tracing::{debug, error, info, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use tracing::Level;

const HISTORY_FILE: &str = "kana_history.json";

fn setup_logging() -> Result<()> {
    // Set up file appender
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "logs",
        "kana_practice.log",
    );

    // Set different log levels based on build type
    let env_filter = if cfg!(debug_assertions) {
        // Debug build - include debug and higher
        EnvFilter::new("debug")
    } else {
        // Release build - include info and higher
        EnvFilter::new("info")
    };

    // Initialize subscriber with configured filter
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(file_appender)
        .with_ansi(false)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .init();

    info!("Logging system initialized");
    debug!("Debug logging {}", if cfg!(debug_assertions) { "enabled" } else { "disabled" });
    
    Ok(())
}

fn parse_practice_mode(arg: &str) -> Option<PracticeMode> {
    // Remove leading dashes
    let arg = arg.trim_start_matches('-');
    
    // Convert to lowercase for case-insensitive matching
    let arg = arg.to_lowercase();
    
    match arg.as_str() {
        // Main mode
        "m" | "ma" | "mai" | "main" => {
            info!("Selected Main practice mode from argument '{}'", arg);
            Some(PracticeMode::Main)
        },
        
        // Dakuten mode
        "d" | "da" | "dak" | "daku" | "dakuten" => {
            info!("Selected Dakuten practice mode from argument '{}'", arg);
            Some(PracticeMode::Dakuten)
        },
        
        // Combination mode
        "c" | "co" | "com" | "comb" | "combo" | "combination" => {
            info!("Selected Combination practice mode from argument '{}'", arg);
            Some(PracticeMode::Combination)
        },
        
        // All mode
        "a" | "al" | "all" => {
            info!("Selected All practice mode from argument '{}'", arg);
            Some(PracticeMode::All)
        },
        
        // No match
        _ => None
    }
}

fn parse_args() -> PracticeMode {
    let args: Vec<String> = env::args().skip(1).collect();
    
    if args.is_empty() {
        info!("No practice mode specified, defaulting to Main");
        return PracticeMode::Main;
    }

    let first_arg = &args[0];
    
    match parse_practice_mode(first_arg) {
        Some(mode) => mode,
        None => {
            warn!("Invalid practice mode '{}', defaulting to Main", first_arg);
            PracticeMode::Main
        }
    }
}
fn main() -> Result<()> {
    setup_logging()?;
    info!("Starting kana practice application");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.set_practice_mode(parse_args());
    match load_history(&mut app) {
        Ok(_) => info!("Successfully loaded history"),
        Err(e) => warn!("Failed to load history: {}", e),
    }

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

    if let Err(e) = save_history(&app) {
        error!("Failed to save history: {}", e);
    } else {
        info!("Successfully saved history");
    }

    if let Err(err) = res {
        error!("Application error: {}", err);
        println!("Error: {}", err);
    }

    info!("Application terminated");
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
                    }
                    KeyCode::Enter => {
                        app.handle_enter()?;
                    }
                    KeyCode::Backspace => {
                        if app.state.mode == AppMode::Ready {
                            app.state.input_buffer.pop();
                        }
                    }
                    KeyCode::Esc => {
                        app.should_quit = true;
                    }
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
        
        for (kana, stats) in app.state.history.character_stats.iter_mut() {
            let stored_ema_response = stats.exp_avg_response;
            let stored_ema_accuracy = stats.exp_avg_accuracy;
            
            stats.recalculate_ema();
            
            if (stats.exp_avg_response - stored_ema_response).abs() > 1e-10 ||
               (stats.exp_avg_accuracy - stored_ema_accuracy).abs() > 1e-10 {
                warn!(
                    kana = kana,
                    stored_response = stored_ema_response,
                    stored_accuracy = stored_ema_accuracy,
                    recalculated_response = stats.exp_avg_response,
                    recalculated_accuracy = stats.exp_avg_accuracy,
                    "EMA mismatch detected"
                );
            }
        }
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
