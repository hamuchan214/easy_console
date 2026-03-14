use regex::Regex;

pub struct SearchState {
    pub query: String,
    pub regex: Option<Regex>,
    pub matches: Vec<usize>, // line indices that match
    pub current_match: Option<usize>,
    #[allow(dead_code)]
    pub filter_mode: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            regex: None,
            matches: Vec::new(),
            current_match: None,
            filter_mode: false,
        }
    }
}

impl SearchState {
    pub fn set_query(&mut self, query: &str) {
        self.query = query.to_string();
        self.regex = Regex::new(query).ok();
        self.matches.clear();
        self.current_match = None;
    }

    pub fn update_matches(&mut self, lines: &[String]) {
        self.matches.clear();
        if let Some(re) = &self.regex {
            for (i, line) in lines.iter().enumerate() {
                if re.is_match(line) {
                    self.matches.push(i);
                }
            }
        }
        if self.matches.is_empty() {
            self.current_match = None;
        } else {
            self.current_match = Some(0);
        }
    }

    pub fn next_match(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.current_match = Some(match self.current_match {
            None => 0,
            Some(i) => (i + 1) % self.matches.len(),
        });
    }

    pub fn prev_match(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.current_match = Some(match self.current_match {
            None => 0,
            Some(0) => self.matches.len() - 1,
            Some(i) => i - 1,
        });
    }

    pub fn current_line_index(&self) -> Option<usize> {
        self.current_match
            .and_then(|i| self.matches.get(i))
            .copied()
    }

    pub fn is_match(&self, line_index: usize) -> bool {
        self.matches.contains(&line_index)
    }
}
