use egui::Context;

pub struct SerialWindow {
    pub output: String,
}

impl SerialWindow {
    pub fn new() -> Self {
        SerialWindow {
            output: "".to_string(),
        }
    }

    pub fn append(&mut self, byte: u8) {
        self.output = format!("{}{}", self.output, byte as char);
    }

    pub fn render(&mut self, ctx: &Context, open: &mut bool) {
        egui::Window::new("Serial Transfer")
            .open(open)
            .resizable(true)
            .show(ctx, |ui| ui.label(&self.output));
    }
}
