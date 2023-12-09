pub struct PVTable {
    pub pv_length: [usize; 64],
    pub pv_table: [[u32; 64]; 64],
}

impl PVTable {
    pub fn new() -> PVTable {
        PVTable {
            pv_length: [0; 64],
            pv_table: [[0; 64]; 64],
        }
    }
}

struct PVEntry {
    key: u64,
    depth: u8,
    value: i32,
}
