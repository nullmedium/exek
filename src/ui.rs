use crate::path_completion::PathCompletion;
use crate::search::SearchResult;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub enum SearchMode {
    Applications(Vec<SearchResult>),
    Paths(Vec<PathCompletion>),
}

pub struct AppState {
    pub query: String,
    pub cursor_position: usize,
    pub selected_index: usize,
    pub mode: SearchMode,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            cursor_position: 0,
            selected_index: 0,
            mode: SearchMode::Applications(Vec::new()),
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        let max_index = match &self.mode {
            SearchMode::Applications(results) => results.len(),
            SearchMode::Paths(completions) => completions.len(),
        };

        if max_index > 0 && self.selected_index < max_index - 1 {
            self.selected_index += 1;
        }
    }

    pub fn reset_selection(&mut self) {
        self.selected_index = 0;
    }

    pub fn get_selected_app(&self) -> Option<&SearchResult> {
        match &self.mode {
            SearchMode::Applications(results) => results.get(self.selected_index),
            _ => None,
        }
    }

    pub fn get_selected_path(&self) -> Option<&PathCompletion> {
        match &self.mode {
            SearchMode::Paths(completions) => completions.get(self.selected_index),
            _ => None,
        }
    }

    pub fn results_count(&self) -> usize {
        match &self.mode {
            SearchMode::Applications(results) => results.len(),
            SearchMode::Paths(completions) => completions.len(),
        }
    }
}

pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    render_search_box(frame, chunks[0], state);
    render_results(frame, chunks[1], state);
}

fn render_search_box(frame: &mut Frame, area: Rect, state: &AppState) {
    let input = Paragraph::new(state.query.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Search "),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(input, area);

    frame.set_cursor_position((
        area.x + 1 + state.cursor_position as u16,
        area.y + 1,
    ));
}

fn render_results(frame: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = match &state.mode {
        SearchMode::Applications(results) => {
            results
                .iter()
                .enumerate()
                .map(|(i, result)| {
                    let is_selected = i == state.selected_index;

                    let mut spans = vec![
                        Span::styled(
                            &result.app.name,
                            if is_selected {
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::White)
                            },
                        ),
                    ];

                    if let Some(comment) = &result.app.comment {
                        spans.push(Span::styled(
                            format!(" - {}", comment),
                            Style::default().fg(Color::Gray),
                        ));
                    }

                    if result.frecency > 0.0 {
                        spans.push(Span::styled(
                            format!(" [{}]", result.app.categories.first().unwrap_or(&String::new())),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }

                    ListItem::new(Line::from(spans))
                })
                .collect()
        },
        SearchMode::Paths(completions) => {
            completions
                .iter()
                .enumerate()
                .map(|(i, completion)| {
                    let is_selected = i == state.selected_index;

                    let mut spans = vec![
                        Span::styled(
                            if completion.is_dir { "📁 " } else { "🔧 " },
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(
                            &completion.display_name,
                            if is_selected {
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::White)
                            },
                        ),
                    ];

                    if completion.is_dir {
                        spans.push(Span::styled(
                            "/",
                            Style::default().fg(Color::Blue),
                        ));
                    }

                    ListItem::new(Line::from(spans))
                })
                .collect()
        },
    };

    let title = match &state.mode {
        SearchMode::Applications(_) => format!(" Applications ({}) ", state.results_count()),
        SearchMode::Paths(_) => format!(" Path Completions ({}) ", state.results_count()),
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray))
                .title(title),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(list, area);
}
