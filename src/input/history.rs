pub struct InputHistory {
    entries: Vec<String>,
    max_size: usize,
    current_index: Option<usize>,
    temp_input: String,
}

impl InputHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_size,
            current_index: None,
            temp_input: String::new(),
        }
    }

    pub fn push(&mut self, entry: String) {
        if entry.is_empty() {
            return;
        }
        // Remove duplicate if exists
        if let Some(pos) = self.entries.iter().position(|e| e == &entry) {
            self.entries.remove(pos);
        }
        self.entries.push(entry);
        if self.entries.len() > self.max_size {
            self.entries.remove(0);
        }
        self.current_index = None;
        self.temp_input.clear();
    }

    pub fn navigate_up(&mut self, current_input: &str) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }
        match self.current_index {
            None => {
                self.temp_input = current_input.to_string();
                self.current_index = Some(self.entries.len() - 1);
            }
            Some(0) => {}
            Some(i) => {
                self.current_index = Some(i - 1);
            }
        }
        self.current_index
            .and_then(|i| self.entries.get(i))
            .map(|s| s.as_str())
    }

    pub fn navigate_down(&mut self) -> Option<&str> {
        match self.current_index {
            None => None,
            Some(i) if i + 1 >= self.entries.len() => {
                self.current_index = None;
                Some(self.temp_input.as_str())
            }
            Some(i) => {
                self.current_index = Some(i + 1);
                self.entries.get(i + 1).map(|s| s.as_str())
            }
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.current_index = None;
        self.temp_input.clear();
    }
}
