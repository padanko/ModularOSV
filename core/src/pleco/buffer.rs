pub struct ViewBuffer {
    pub cursor: usize,
    pub buffer: String,
    pub filename: String,
}

impl ViewBuffer {
    pub fn new(filename: &str) -> Self {
        Self {
            cursor: 0,
            buffer: String::new(),
            filename: filename.into(),
        }
    }

    pub fn add_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn remove_char(&mut self) {
        if self.cursor == 0 || self.buffer.is_empty() {
            return;
        }

        if let Some((prev_idx, _)) = self
            .buffer
            .char_indices()
            .rev()
            .find(|(i, _)| *i < self.cursor)
        {
            self.buffer.remove(prev_idx);
            self.cursor = prev_idx;
        }
    }

    pub fn cur_move_left(&mut self) {
        if self.cursor == 0 {
            return;
        }

        if let Some((prev_idx, _)) = self
            .buffer
            .char_indices()
            .rev()
            .find(|(i, _)| *i < self.cursor)
        {
            self.cursor = prev_idx;
        }
    }

    pub fn cur_move_right(&mut self) {
        if let Some((next_idx, _)) = self.buffer.char_indices().find(|(i, _)| *i > self.cursor) {
            self.cursor = next_idx;
        } else {
            self.cursor = self.buffer.len();
        }
    }
}
