mod database;
mod desktop_entry;
mod path_completion;
mod search;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use database::Database;
use desktop_entry::DesktopScanner;
use path_completion::PathCompleter;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use search::Searcher;
use std::env;
use std::io;
use std::process::Command;
use ui::{AppState, SearchMode};

fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal);
    restore_terminal(&mut terminal)?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let apps = DesktopScanner::scan()?;
    let mut database = Database::load().unwrap_or_else(|_| Database::new());
    let searcher = Searcher::new(database.clone());
    let path_completer = PathCompleter::new();
    let mut state = AppState::new();

    state.mode = SearchMode::Applications(searcher.search("", &apps));

    loop {
        terminal.draw(|frame| ui::render(frame, &state))?;

        if let Event::Key(key) = event::read()? {
            match (key.code, key.modifiers) {
                (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    return Ok(());
                }
                (KeyCode::Tab, _) => {
                    if let SearchMode::Paths(completions) = &state.mode {
                        if let Some(selected) = completions.get(state.selected_index) {
                            state.query = path_completion::PathCompleter::apply_completion(&state.query, selected);
                            state.cursor_position = state.query.len();
                            let new_completions = path_completer.complete_path(&state.query);
                            state.mode = SearchMode::Paths(new_completions);
                            state.reset_selection();
                        }
                    }
                }
                (KeyCode::Enter, _) => {
                    match &state.mode {
                        SearchMode::Applications(_) => {
                            if let Some(selected) = state.get_selected_app() {
                                database.record_launch(&selected.app.name)?;
                                launch_app(&selected.app)?;
                                return Ok(());
                            }
                        }
                        SearchMode::Paths(completions) => {
                            if let Some(selected) = completions.get(state.selected_index) {
                                if selected.is_dir {
                                    state.query = path_completion::PathCompleter::apply_completion(&state.query, selected);
                                    state.cursor_position = state.query.len();
                                    let new_completions = path_completer.complete_path(&state.query);
                                    state.mode = SearchMode::Paths(new_completions);
                                    state.reset_selection();
                                } else {
                                    launch_executable(&selected.path)?;
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                    state.move_selection_up();
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                    state.move_selection_down();
                }
                (KeyCode::Backspace, _) => {
                    if state.cursor_position > 0 {
                        let remove_pos = state.cursor_position - 1;
                        state.query.remove(remove_pos);
                        state.cursor_position -= 1;
                        update_search_mode(&mut state, &searcher, &path_completer, &apps);
                        state.reset_selection();
                    }
                }
                (KeyCode::Delete, _) => {
                    if state.cursor_position < state.query.len() {
                        state.query.remove(state.cursor_position);
                        update_search_mode(&mut state, &searcher, &path_completer, &apps);
                        state.reset_selection();
                    }
                }
                (KeyCode::Left, _) => {
                    if state.cursor_position > 0 {
                        state.cursor_position -= 1;
                    }
                }
                (KeyCode::Right, _) => {
                    if state.cursor_position < state.query.len() {
                        state.cursor_position += 1;
                    }
                }
                (KeyCode::Home, _) => {
                    state.cursor_position = 0;
                }
                (KeyCode::End, _) => {
                    state.cursor_position = state.query.len();
                }
                (KeyCode::Char(c), _) => {
                    state.query.insert(state.cursor_position, c);
                    state.cursor_position += 1;
                    update_search_mode(&mut state, &searcher, &path_completer, &apps);
                    state.reset_selection();
                }
                _ => {}
            }
        }
    }
}

fn update_search_mode(
    state: &mut AppState,
    searcher: &Searcher,
    path_completer: &PathCompleter,
    apps: &[desktop_entry::AppEntry],
) {
    if path_completion::PathCompleter::is_path_query(&state.query) {
        let completions = path_completer.complete_path(&state.query);
        state.mode = SearchMode::Paths(completions);
    } else {
        let results = searcher.search(&state.query, apps);
        state.mode = SearchMode::Applications(results);
    }
}

fn launch_app(app: &desktop_entry::AppEntry) -> Result<()> {
    let cmd = app.get_launch_command();
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }

    // Resolve the executable to its absolute path if possible
    let executable = parts[0];
    let resolved_executable = if executable.contains('/') {
        // Already a path, expand it
        shellexpand::tilde(executable).to_string()
    } else {
        // Try to find in PATH
        which::which(executable)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| executable.to_string())
    };

    let mut command = Command::new(&resolved_executable);

    // Ensure proper environment variables
    ensure_environment(&mut command);

    // Add arguments
    for arg in &parts[1..] {
        command.arg(arg);
    }

    if app.terminal {
        let terminal_emulators = ["x-terminal-emulator", "gnome-terminal", "konsole", "xterm", "alacritty", "kitty"];
        for term in &terminal_emulators {
            if let Ok(term_path) = which::which(term) {
                command = Command::new(term_path);
                ensure_environment(&mut command);
                command.arg("-e");
                command.arg(&cmd);
                break;
            }
        }
    }

    // Use spawn with detached process
    use std::os::unix::process::CommandExt;
    command.stdin(std::process::Stdio::null());
    command.stdout(std::process::Stdio::null());
    command.stderr(std::process::Stdio::null());

    // Start new process group (detach from parent)
    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    command.spawn()?;
    Ok(())
}

fn ensure_environment(command: &mut Command) {
    // Ensure PATH includes common directories
    let path = env::var("PATH").unwrap_or_else(|_| String::new());
    let default_paths = vec![
        "/usr/local/sbin",
        "/usr/local/bin",
        "/usr/sbin",
        "/usr/bin",
        "/sbin",
        "/bin",
        "/usr/games",
        "/usr/local/games",
        "/snap/bin",
    ];

    let mut path_parts: Vec<String> = path.split(':').map(|s| s.to_string()).collect();

    for default_path in default_paths {
        if !path_parts.iter().any(|p| p == default_path) {
            path_parts.push(default_path.to_string());
        }
    }

    command.env("PATH", path_parts.join(":"));

    // Ensure DISPLAY is set for GUI apps
    if let Ok(display) = env::var("DISPLAY") {
        command.env("DISPLAY", display);
    } else {
        command.env("DISPLAY", ":0");
    }

    // Ensure XDG_RUNTIME_DIR is set
    if let Ok(xdg_runtime) = env::var("XDG_RUNTIME_DIR") {
        command.env("XDG_RUNTIME_DIR", xdg_runtime);
    } else if let Ok(uid) = env::var("UID") {
        command.env("XDG_RUNTIME_DIR", format!("/run/user/{}", uid));
    }

    // Pass through WAYLAND_DISPLAY if set
    if let Ok(wayland) = env::var("WAYLAND_DISPLAY") {
        command.env("WAYLAND_DISPLAY", wayland);
    }

    // Set HOME if not set
    if env::var("HOME").is_err() {
        if let Some(home) = dirs::home_dir() {
            command.env("HOME", home);
        }
    }
}

fn launch_executable(path: &std::path::Path) -> Result<()> {
    let mut command = Command::new(path);

    // Ensure proper environment
    ensure_environment(&mut command);

    // Detach from parent
    use std::os::unix::process::CommandExt;
    command.stdin(std::process::Stdio::null());
    command.stdout(std::process::Stdio::null());
    command.stderr(std::process::Stdio::null());

    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    command.spawn()?;
    Ok(())
}
