use super::Background;
impl super::Window {
    pub fn set_background_color(&mut self, color: u32) {
            self.background = Some(Background::Color(color));
        }

    pub fn set_background_image(&mut self, image: Vec<u32>) {
        assert_eq!(image.len(), self.width * self.height, "Background image must match window size");
        self.background = Some(Background::Image(image));
    }
}