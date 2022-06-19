extern crate ctrlc;
extern crate num_traits;
extern crate png;
extern crate winit;

#[macro_use]
pub mod macros;

pub mod conv;
pub mod core;
pub mod debug;
pub mod gameboy;
pub mod test_runner;
pub mod ui;
pub mod utils;
pub mod wave_audio_recorder;

pub const APPNAME: &str = "Rustboy?";
pub const VERSION: &str = "0.0.0";
pub const AUTHOR: &str = "Jonatan Magnusson <jonatan.magnusson@gmail.com>";
