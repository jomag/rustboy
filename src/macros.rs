
use debug::{print_registers};

#[macro_export]
macro_rules! panic_debug {
    ($msg:expr, $mmu:expr) => {
        println!("Panic! {}", $msg);
        print_registers($mmu);
        print_ppu_registers($mmu);
        panic!()
    }
}

