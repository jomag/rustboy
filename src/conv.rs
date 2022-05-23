pub const KIB: usize = 1_024;
pub const MIB: usize = 1_048_576;

pub fn kib(n: usize) -> usize {
    return n * KIB;
}

#[allow(dead_code)]
pub fn mib(n: usize) -> usize {
    return n * MIB;
}
