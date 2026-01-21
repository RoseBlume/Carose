use crate::{Window, TextAlign};

pub struct Menu {
    pub options: Vec<&'static str>,
    selected: usize,
    selected_col: u32,
    unselected: u32
}

impl Menu {
    pub fn new(options: Vec<&'static str>, selected_col: u32, unselected: u32) -> Self {
        Self { options, selected: 0, selected_col, unselected }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 { self.selected -= 1; }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.options.len() { self.selected += 1; }
    }

    pub fn current(&self) -> &str {
        self.options[self.selected]
    }

    pub fn draw(&self, window: &mut Window, id_prefix: &str) {
        // Remove previous text for this menu
        for i in 0..self.options.len() {
            let id = format!("{}_{}", id_prefix, i);
            window.remove_text(&id);
        }

        let start_y = 200;
        let gap = 50;
        for (i, option) in self.options.iter().enumerate() {
            let color = if i == self.selected { self.selected_col } else { self.unselected };
            window.show_text(
                &format!("{}_{}", id_prefix, i),
                option,
                (350, start_y + i * gap),
                5,
                color,
                TextAlign::AutoFit,
            );
        }
    }
}