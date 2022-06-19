use std::usize::MAX;
use std::{iter, sync::Arc, time::Instant};

use crate::core::Core;
use crate::debug::Debug;

use crate::gameboy::emu::Emu;
use crate::gameboy::CLOCK_SPEED;
use crate::APPNAME;

use egui::{FontDefinitions, Key, Label};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use epi::*;
use ringbuf::{Consumer, RingBuffer};
use wgpu::{Device, FilterMode, Queue, Surface, SurfaceConfiguration};

use super::app::MoeApp;
use super::gameboy;
use super::gameboy::main_window::MainWindow;
use super::{audio_player::AudioPlayer, render_stats::RenderStats};
