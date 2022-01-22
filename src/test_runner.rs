use crate::{debug::Debug, emu::Emu};

pub fn test_runner(variant: &str, emu: &mut Emu) {
    match variant {
        "mooneye" => {
            // When tests in the Mooneye test suite finishes successfully:
            //
            // - Registers B, C, D, E, H, L will hold Fibonacci sequence 3, 5, 8,
            //   13, 21, 34.
            // - Software breakpoint will be triggered ("ld b,b")
            // - The same Fibonacci sequence will be sent over serial
            //
            // When tests in the Mooneye test suite fails:
            //
            // - Software breakpoint will be triggered ("ld b,b")
            // - Registers will *not* hold the Fibonacci sequence
            // - 0x42 ("B") will be sent 6 times over serial
            //
            // Details: https://github.com/Gekkio/mooneye-test-suite
            let debug = Debug {
                source_code_breakpoints: true,
            };

            while !debug.check_breakpoints(emu) {
                emu.mmu.exec_op();
            }

            // Verify that registers hold Fibonacci sequence
            let reg = &emu.mmu.reg;
            if reg.b == 3 && reg.c == 5 && reg.d == 8 && reg.e == 13 && reg.h == 21 && reg.l == 34 {
                println!("Fibonacci registers: ok!");
            } else {
                println!("Fibonacci registers: fail!");
                println!(
                    "B: 0x{:02x} C: 0x{:02x} D: 0x{:02x} E: 0x{:02x} H: 0x{:02x} L: 0x{:02x}",
                    reg.b, reg.c, reg.d, reg.e, reg.h, reg.l
                );
                std::process::exit(1);
            }

            println!("PASS!");
            std::process::exit(0);
        }
        _ => {
            println!("Unknown test runner variant: {}", variant);
            println!("Currently supported variants:");
            println!(" - mooneye");
            std::process::exit(1);
        }
    }
}
