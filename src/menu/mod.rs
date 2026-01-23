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

        let (width, height) = window.get_size();

        // Calculate starting Y so the menu is vertically centered
        let gap = 50;
        let total_height = gap * (self.options.len() - 1);
        let start_y = (height / 2).saturating_sub(total_height / 2);

        for (i, option) in self.options.iter().enumerate() {
            let color = if i == self.selected { self.selected_col } else { self.unselected };
            let text_width = option.len() * 5 * 5; // rough width approximation for AutoFit 5x5 font scaled by size
            let x = width / 2 - text_width / 2;    // center text horizontally
            let y = start_y + i * gap;

            window.show_text(
                &format!("{}_{}", id_prefix, i),
                option,
                (x, y),
                5,
                color,
                TextAlign::AutoFit,
            );
        }
    }
}