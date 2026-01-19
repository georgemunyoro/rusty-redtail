pub mod board;
pub mod chess;
pub mod movegen;
pub mod pst;
pub mod search;
pub mod tt;
pub mod utils;

#[derive(Debug, Copy, Clone)]
pub struct Cutoffs {
    pub total: u32,
    pub move_1: u32,
    pub move_2: u32,
    pub avg_cutoff_move_no: f32,
}

impl Cutoffs {
    pub fn new() -> Self {
        Self {
            total: 0,
            move_1: 0,
            move_2: 0,
            avg_cutoff_move_no: 0.0,
        }
    }
}
