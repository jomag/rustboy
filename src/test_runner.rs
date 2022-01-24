use ringbuf::RingBuffer;

use crate::{
    debug::{format_mnemonic, Debug},
    emu::Emu,
    utils::read_zero_terminated_string,
};

pub fn test_runner_expect(expect: &str, emu: &mut Emu, debug: &mut Debug) {
    let echo_serial: bool = false;
    let mut output: String = "".to_string();
    let serial_buf = RingBuffer::<u8>::new(16);
    let (producer, mut consumer) = serial_buf.split();
    emu.mmu.serial.output = Some(producer);

    loop {
        emu.mmu.exec_op();

        match consumer.pop() {
            Some(c) => {
                output.push(c as char);
                if echo_serial {
                    print!("{}", c as char);
                }
            }
            None => {}
        }

        if output == expect || output.len() > expect.len() {
            break;
        }
    }

    if output != expect {
        println!("Actual: {:?}", output.as_bytes());
        println!("Expected: {:?}", expect.as_bytes());
        std::process::exit(1);
    }

    if echo_serial {
        println!();
    }
    std::process::exit(0);
}

pub fn test_runner(variant: &str, emu: &mut Emu, debug: &mut Debug) {
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
            debug.source_code_breakpoints = true;

            while !debug.before_op(emu) {
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

        "blargg" => {
            // For Blargg sound tests, address 0xA000 holds the general status:
            // if the tests is still running, it's value is 0x80. When it
            // finishes it holds the result code.
            //
            // 0xA001..0xA003 holds the signature values 0xDE, 0xB0, 0x61.
            //
            // Text output is available as a zero-terminated string at 0xA004.

            // First run until 0xA000 is 0x80
            while emu.mmu.direct_read(0xA000) != 0x80 {
                // println!(
                //     "PC = {:04x} 0xA000 = {}, 0xFF01={}, 0xFF02={} {}",
                //     emu.mmu.reg.pc,
                //     emu.mmu.direct_read(0xA000),
                //     emu.mmu.direct_read(0xFF01),
                //     emu.mmu.direct_read(0xFF02),
                //     format_mnemonic(&emu.mmu, emu.mmu.reg.pc),
                // );
                debug.before_op(emu);
                emu.mmu.exec_op();
            }

            // Then run until test finishes (0xA000 != 0x80)
            while emu.mmu.direct_read(0xA000) == 0x80 {
                // println!("0xA000 = {}", emu.mmu.direct_read(0xA000));
                // debug.before_op(emu);
                emu.mmu.exec_op();
            }

            // Validate signature
            let sig1 = emu.mmu.direct_read(0xA001);
            let sig2 = emu.mmu.direct_read(0xA002);
            let sig3 = emu.mmu.direct_read(0xA003);

            if sig1 != 0xDE || sig2 != 0xB0 || sig3 != 0x61 {
                println!(
                    "Invalid signature: 0x{:02x} 0x{:02x} 0x{:02x}",
                    sig1, sig2, sig3
                );
                std::process::exit(1);
            }

            // Read result string
            let result_string = read_zero_terminated_string(&emu.mmu, 0xA004);
            match result_string {
                Ok(s) => println!("Result string: {}", s),
                Err(e) => println!("Failed to read result string: {}", e),
            };

            // Result code. 0 = success.
            let result_code = emu.mmu.direct_read(0xA000);
            match result_code {
                0x00 => {
                    println!("Result code: 0x{:02x}: success!", result_code);
                    std::process::exit(0);
                }
                n => {
                    println!("Result code: 0x{:02x}: fail!", n);
                    std::process::exit(1);
                }
            }
        }

        _ => {
            println!("Unknown test runner variant: {}", variant);
            println!("Currently supported variants:");
            println!(" - mooneye");
            println!(" - blargg");
            std::process::exit(1);
        }
    }
}
