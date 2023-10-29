pub static STARTING_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub static FILE_A: &'static u64 = &0x0101010101010101;
pub static FILE_H: &'static u64 = &0x8080808080808080;

pub static FILE_GH: &'static u64 = &0xC0C0C0C0C0C0C0C0;
pub static FILE_AB: &'static u64 = &0x0303030303030303;

pub static RANK_8: &'static u64 = &0x00000000000000FF;
pub static RANK_7: &'static u64 = &0x000000000000FF00;
pub static RANK_6: &'static u64 = &0x0000000000FF0000;
pub static RANK_5: &'static u64 = &0x00000000FF000000;
pub static RANK_4: &'static u64 = &0x000000FF00000000;
pub static RANK_2: &'static u64 = &0x00FF000000000000;
pub static RANK_1: &'static u64 = &0xFF00000000000000;
