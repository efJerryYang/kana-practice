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

const HISTORY_FILE: &str = "kana_history.json";
const VALID_PRACTICE_FLAGS: &[&str] = &["main", "dakuten", "combination"];
const VALID_KANA_FLAGS: &[&str] = &["hiragana", "katakana"];

#[derive(Debug)]
struct CliArgs {
    kana_type: KanaType,
    practice_type: PracticeType,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum KanaType {
    Hiragana,
    Katakana,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum PracticeType {
    Main,
    Dakuten,
    Combination,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            kana_type: KanaType::Hiragana,
            practice_type: PracticeType::Main,
        }
    }
}

fn setup_logging() -> Result<()> {
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "logs",
        "kana_practice.log",
    );

    let env_filter = if cfg!(debug_assertions) {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

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

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();
    
    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }
    
    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];
    
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }
    
    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            matrix[i + 1][j + 1] = [
                matrix[i][j + 1] + 1,
                matrix[i + 1][j] + 1,
                matrix[i][j] + cost,
            ].iter().min().unwrap().clone();
        }
    }
    
    matrix[a_len][b_len]
}

fn find_closest_match<'a>(input: &'a str, valid_options: &'a [&'a str]) -> Option<&'a str> {
    valid_options
        .iter()
        .filter(|&&option| option.starts_with(&input[..1]))
        .min_by_key(|&&option| levenshtein_distance(input, option))
        .copied()
}

fn is_valid_prefix<'a>(arg: &'a str, valid_flags: &'a [&'a str]) -> Option<&'a str> {
    valid_flags.iter()
        .find(|&&flag| flag.starts_with(arg))
        .copied()
}

fn parse_kana_type(arg: &str) -> Option<KanaType> {
    let arg = arg.trim_start_matches('-').to_lowercase();
    
    if let Some(matched_flag) = is_valid_prefix(&arg, VALID_KANA_FLAGS) {
        return match matched_flag {
            "hiragana" => Some(KanaType::Hiragana),
            "katakana" => Some(KanaType::Katakana),
            _ => None,
        };
    }
    
    if let Some(suggestion) = find_closest_match(&arg, VALID_KANA_FLAGS) {
        warn!(
            "Unknown kana type '{}'. Did you mean '--{}'?",
            arg, suggestion
        );
    } else {
        warn!("Unknown kana type '{}'. Valid options are: {:?}", arg, VALID_KANA_FLAGS);
    }
    None
}

fn parse_practice_type(arg: &str) -> Option<PracticeType> {
    let arg = arg.trim_start_matches('-').to_lowercase();
    
    if let Some(matched_flag) = is_valid_prefix(&arg, VALID_PRACTICE_FLAGS) {
        return match matched_flag {
            "main" => Some(PracticeType::Main),
            "dakuten" => Some(PracticeType::Dakuten),
            "combination" => Some(PracticeType::Combination),
            _ => None,
        };
    }
    
    if let Some(suggestion) = find_closest_match(&arg, VALID_PRACTICE_FLAGS) {
        warn!(
            "Unknown practice type '{}'. Did you mean '--{}'?",
            arg, suggestion
        );
    } else {
        warn!(
            "Unknown practice type '{}'. Valid options are: {:?}",
            arg, VALID_PRACTICE_FLAGS
        );
    }
    None
}

fn parse_combined_flags(flags: &str) -> Option<(Option<PracticeType>, Option<KanaType>)> {
    if flags.len() != 2 {
        return None;
    }
    
    let (first, second) = (flags.chars().nth(0)?, flags.chars().nth(1)?);
    
    let try_order = |c1, c2| {
        let practice = match c1 {
            'm' => Some(PracticeType::Main),
            'd' => Some(PracticeType::Dakuten),
            'c' => Some(PracticeType::Combination),
            _ => None,
        };
        
        let kana = match c2 {
            'h' => Some(KanaType::Hiragana),
            'k' => Some(KanaType::Katakana),
            _ => None,
        };
        
        if practice.is_some() || kana.is_some() {
            Some((practice, kana))
        } else {
            None
        }
    };
    
    try_order(first, second).or_else(|| try_order(second, first))
}



fn parse_args() -> Result<CliArgs> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut cli_args = CliArgs::default();
    let mut practice_type_set = false;
    let mut kana_type_set = false;

    if args.is_empty() {
        info!("No arguments provided, falling back to default settings: main hiragana. Available options:");
        info!("Practice types: {:?}", VALID_PRACTICE_FLAGS);
        info!("Kana types: {:?}", VALID_KANA_FLAGS);
        return Ok(cli_args);
    }

    for arg in args {
        if arg.is_empty() {
            continue;
        }

        // Handle single short flag (-h, -m etc)
        if arg.starts_with('-') && !arg.starts_with("--") && arg.len() == 2 {
            let flag = &arg[1..];
            // Try as kana type
            if let Some(kana_type) = match flag {
                "h" => Some(KanaType::Hiragana),
                "k" => Some(KanaType::Katakana),
                _ => None,
            } {
                cli_args.kana_type = kana_type;
                kana_type_set = true;
                continue;
            }
            // Try as practice type
            if let Some(practice_type) = match flag {
                "m" => Some(PracticeType::Main),
                "d" => Some(PracticeType::Dakuten),
                "c" => Some(PracticeType::Combination),
                _ => None,
            } {
                cli_args.practice_type = practice_type;
                practice_type_set = true;
                continue;
            }
            return Err(KanaError::InvalidInput(format!(
                "Invalid short flag: '{}'. Valid short flags are: h, k, m, d, c",
                flag
            )));
        }

        // Handle combined short flags (-mh, -hm etc)
        if arg.starts_with('-') && !arg.starts_with("--") && arg.len() > 2 {
            let flags = &arg[1..];
            if let Some((practice, kana)) = parse_combined_flags(flags) {
                if let Some(p) = practice {
                    cli_args.practice_type = p;
                    practice_type_set = true;
                }
                if let Some(k) = kana {
                    cli_args.kana_type = k;
                    kana_type_set = true;
                }
                continue;
            } else {
                return Err(KanaError::InvalidInput(format!(
                    "Invalid combined flags: '{}'. Valid combinations are: mh, hm, mk, km, dh, hd, dk, kd, ch, hc, ck, kc",
                    flags
                )));
            }
        }

        // Handle long flags (--hiragana, --main etc)
        if let Some(kana_type) = parse_kana_type(&arg) {
            cli_args.kana_type = kana_type;
            kana_type_set = true;
            continue;
        }

        if let Some(practice_type) = parse_practice_type(&arg) {
            cli_args.practice_type = practice_type;
            practice_type_set = true;
            continue;
        }

        // If we get here, the argument is unknown
        if let Some(suggestion) = find_closest_match(&arg.trim_start_matches('-'), VALID_PRACTICE_FLAGS) {
            return Err(KanaError::InvalidInput(format!(
                "Unknown argument: '{}'. Did you mean '--{}'?",
                arg, suggestion
            )));
        } else if let Some(suggestion) = find_closest_match(&arg.trim_start_matches('-'), VALID_KANA_FLAGS) {
            return Err(KanaError::InvalidInput(format!(
                "Unknown argument: '{}'. Did you mean '--{}'?",
                arg, suggestion
            )));
        } else {
            return Err(KanaError::InvalidInput(format!(
                "Unknown argument: '{}'. Valid options are:\nPractice types: {:?}\nKana types: {:?}",
                arg, VALID_PRACTICE_FLAGS, VALID_KANA_FLAGS
            )));
        }
    }

    info!(
        kana_type = ?cli_args.kana_type,
        kana_type_set = kana_type_set,
        practice_type = ?cli_args.practice_type,
        practice_type_set = practice_type_set,
        "Parsed CLI arguments"
    );

    if cli_args.kana_type == KanaType::Katakana {
        return Err(KanaError::InvalidInput(
            "Katakana practice is not yet implemented".to_string()
        ));
    }

    Ok(cli_args)
}


fn convert_to_practice_mode(cli_args: &CliArgs) -> PracticeMode {
    match cli_args.practice_type {
        PracticeType::Main => PracticeMode::Main,
        PracticeType::Dakuten => PracticeMode::Dakuten,
        PracticeType::Combination => PracticeMode::Combination,
    }
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

fn main() -> Result<()> {
    setup_logging()?;
    info!("Starting kana practice application");

    let cli_args = parse_args()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.set_practice_mode(convert_to_practice_mode(&cli_args));
    
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