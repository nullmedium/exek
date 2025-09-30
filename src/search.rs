use crate::database::Database;
use crate::desktop_entry::AppEntry;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub struct SearchResult {
    pub app: AppEntry,
    pub score: i64,
    pub frecency: f64,
}

pub struct Searcher {
    matcher: SkimMatcherV2,
    database: Database,
}

impl Searcher {
    pub fn new(database: Database) -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
            database,
        }
    }

    pub fn search(&self, query: &str, apps: &[AppEntry]) -> Vec<SearchResult> {
        if query.is_empty() {
            return self.get_recent_apps(apps, 20);
        }

        let mut results: Vec<SearchResult> = apps
            .iter()
            .filter_map(|app| {
                let name_score = self.matcher.fuzzy_match(&app.name, query).unwrap_or(0);

                let exec_base = app.exec.split_whitespace().next().unwrap_or("");
                let exec_name = exec_base.split('/').last().unwrap_or("");
                let exec_score = self.matcher.fuzzy_match(exec_name, query).unwrap_or(0);

                let comment_score = app
                    .comment
                    .as_ref()
                    .and_then(|c| self.matcher.fuzzy_match(c, query))
                    .unwrap_or(0);

                let category_score = app
                    .categories
                    .iter()
                    .filter_map(|cat| self.matcher.fuzzy_match(cat, query))
                    .max()
                    .unwrap_or(0);

                let base_score = name_score.max(exec_score).max(comment_score / 2).max(category_score / 3);

                if base_score > 0 {
                    // Check both regular name and path-based entries
                    let frecency = if app.name.ends_with(" [Path]") {
                        self.database.calculate_frecency(&format!("path:{}", app.exec))
                    } else {
                        self.database.calculate_frecency(&app.name)
                    };
                    let boost = if name_score == base_score { 10 } else { 0 };
                    let final_score = base_score + boost + (frecency.min(100.0) as i64 / 10);

                    Some(SearchResult {
                        app: app.clone(),
                        score: final_score,
                        frecency,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| b.frecency.partial_cmp(&a.frecency).unwrap_or(std::cmp::Ordering::Equal))
                .then_with(|| a.app.name.cmp(&b.app.name))
        });

        results
    }

    fn get_recent_apps(&self, apps: &[AppEntry], limit: usize) -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = apps
            .iter()
            .map(|app| {
                // Check both regular name and path-based entries
                let frecency = if app.name.ends_with(" [Path]") {
                    self.database.calculate_frecency(&format!("path:{}", app.exec))
                } else {
                    self.database.calculate_frecency(&app.name)
                };
                SearchResult {
                    app: app.clone(),
                    score: frecency as i64,
                    frecency,
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.frecency
                .partial_cmp(&a.frecency)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.app.name.cmp(&b.app.name))
        });

        results.truncate(limit);
        results
    }
}
