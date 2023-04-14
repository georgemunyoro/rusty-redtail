const WHITE: u8 = 0;
const BLACK: u8 = 1;

enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

const PIECE_NAMES: [&str; 6] = ["Pawn", "Knight", "Bishop", "Rook", "Queen", "King"];

const FILE_A: u64 = 0x0101010101010101;
const FILE_B: u64 = 0x0202020202020202;
const FILE_C: u64 = 0x0404040404040404;
const FILE_D: u64 = 0x0808080808080808;
const FILE_E: u64 = 0x1010101010101010;
const FILE_F: u64 = 0x2020202020202020;
const FILE_G: u64 = 0x4040404040404040;
const FILE_H: u64 = 0x8080808080808080;

const RANK_8: u64 = 0x00000000000000FF;
const RANK_7: u64 = 0x000000000000FF00;
const RANK_6: u64 = 0x0000000000FF0000;
const RANK_5: u64 = 0x00000000FF000000;
const RANK_4: u64 = 0x000000FF00000000;
const RANK_3: u64 = 0x0000FF0000000000;
const RANK_2: u64 = 0x00FF000000000000;
const RANK_1: u64 = 0xFF00000000000000;
