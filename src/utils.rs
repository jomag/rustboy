use crate::gameboy::mmu::MMU;

pub fn read_zero_terminated_string(
    mmu: &MMU,
    mut adr: usize,
) -> Result<String, std::str::Utf8Error> {
    let mut c = mmu.direct_read(adr);
    let mut zstr = vec![];
    while c != 0 {
        zstr.push(c);
        adr = adr + 1;
        c = mmu.direct_read(adr);
    }

    match std::str::from_utf8(&zstr) {
        Ok(s) => Ok(String::from(s)),
        Err(e) => Err(e),
    }
}

pub trait VecExt<T> {
    fn push_if(&mut self, cond: bool, val: T);
}

impl<T> VecExt<T> for Vec<T> {
    fn push_if(&mut self, cond: bool, val: T) {
        if cond {
            self.push(val);
        }
    }
}
