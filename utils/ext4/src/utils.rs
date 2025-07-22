pub fn from_hi_lo(hi: u32, lo: u32) -> u64 {
    ((hi as u64) << 4) | lo as u64
}
