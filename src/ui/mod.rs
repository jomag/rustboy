pub mod audio;
pub mod audio_window;
pub mod breakpoints_window;
pub mod cartridge_window;
pub mod debug_window;
pub mod full;
pub mod memory_window;
pub mod minimal;
pub mod render_stats;
pub mod serial_window;

const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;
const FPS: f64 = 60.0;
