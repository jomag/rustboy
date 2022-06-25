use crate::core::Core;
use std::collections::HashMap;

#[derive(PartialEq)]
pub enum ExecState {
    // Continuous execution
    RUN,

    // Continue execution after a breakpoint.
    // State will change to RUN after next operation.
    CONTINUE,

    // Single-stepping
    STEP,
}

pub struct Breakpoint {
    pub enabled: bool,
}

impl Breakpoint {
    pub fn evaluate(&self, _core: &impl Core) -> bool {
        self.enabled
    }
}

pub struct Debug {
    // If true, execution will break on "software breakpoints",
    // aka "ld b, b" instructions (0x40).
    pub source_code_breakpoints: bool,
    pub debug_log: Option<std::fs::File>,
    pub state: ExecState,

    // When single-stepping, steps holds the number of steps
    // queued for execution.
    pub steps: u32,

    pub breakpoints: HashMap<usize, Vec<Breakpoint>>,

    // Execution will break when this scanline is reached.
    // Set to a value >153 to disable.
    pub break_on_scanline: Option<usize>,
}

impl Debug {
    pub fn new() -> Self {
        Debug {
            source_code_breakpoints: false,
            debug_log: None,
            state: ExecState::RUN,
            steps: 0,
            breakpoints: HashMap::new(),
            break_on_scanline: None,
        }
    }

    pub fn add_breakpoint(&mut self, adr: usize, bp: Breakpoint) {
        self.breakpoints.entry(adr).or_insert(vec![]).push(bp);
    }

    pub fn break_on_scanline(&mut self, scanline: usize) {
        self.break_on_scanline = Some(scanline);
    }

    pub fn break_execution(&mut self) {
        println!("Breaking execution");
        self.state = ExecState::STEP;
        self.steps = 0;
    }

    pub fn continue_execution(&mut self) {
        println!("Continue execution");
        self.state = ExecState::CONTINUE;
    }

    pub fn next(&mut self) -> bool {
        match self.state {
            ExecState::RUN => true,
            ExecState::CONTINUE => {
                self.state = ExecState::RUN;
                true
            }
            ExecState::STEP => {
                if self.steps > 0 {
                    self.steps -= 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.steps += 1;
    }

    pub fn start_debug_log(&mut self, filename: &str) {
        self.debug_log = Some(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(filename)
                .unwrap(),
        );
    }

    #[allow(dead_code)]
    pub fn finalize(&mut self) {
        match self.debug_log {
            Some(ref mut f) => match f.sync_all() {
                Ok(_) => {}
                Err(e) => println!("Failed to sync log: {:?}", e),
            },
            None => {}
        };
    }

    // Perform debugging actions before every op.
    // Returns true if a breakpoint has been triggered.
    pub fn before_op(&mut self, core: &impl Core) -> bool {
        // FIXME: this will be executed even if next op is not executed
        // because execution is stopped.
        match self.debug_log {
            Some(ref mut f) => core.log_state(f),
            None => {}
        }

        // Check breakpoints, unless current state is CONTINUE
        // which means that we're continuing after a breakpoint
        // was reached.
        if self.state != ExecState::CONTINUE {
            let pc = core.op_offset();
            if self.breakpoints.contains_key(&pc) {
                for bp in self.breakpoints[&pc].iter() {
                    if bp.evaluate(core) {
                        self.state = ExecState::STEP;
                    }
                }
            }

            if self.source_code_breakpoints && core.at_source_code_breakpoint() {
                self.state = ExecState::STEP;
            }

            match self.break_on_scanline {
                Some(n) => {
                    if core.scanline() == n {
                        self.break_on_scanline = None;
                        self.state = ExecState::STEP;
                    }
                }
                None => {}
            }
        }

        return self.next();
    }
}

#[allow(dead_code)]
pub fn address_type(addr: u16) -> String {
    if addr < 0x4000 {
        return "ROM bank #0".to_string();
    }

    if addr < 0x8000 {
        return "ROM bank #1 (switchable)".to_string();
    }

    if addr < 0xA000 {
        return "Video RAM".to_string();
    }

    if addr < 0xC000 {
        return "Switchable RAM bank".to_string();
    }

    if addr < 0xE000 {
        return "Internal RAM (1)".to_string();
    }

    if addr < 0xFE00 {
        return "Echo of internal RAM".to_string();
    }

    if addr < 0xFEA0 {
        return "Sprite Attrib Memory (OAM)".to_string();
    }

    if addr < 0xFF00 {
        return "Empty memory block, unusable for I/O (1)".to_string();
    }

    if addr < 0xFF4C {
        return "I/O ports".to_string();
    }

    if addr < 0xFF80 {
        return "Empty memory block, unusable for I/O (2)".to_string();
    }

    if addr < 0xFFFF {
        return "Internal RAM (2)".to_string();
    }

    return "Interrupt Enable Register".to_string();
}
