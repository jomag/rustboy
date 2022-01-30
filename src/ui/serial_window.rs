use egui::CtxRef;

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

    pub fn render(&mut self, ctx: &CtxRef) {
        egui::Window::new("Serial Transfer")
            .resizable(true)
            .show(ctx, |ui| ui.label(&self.output));
    }
}
