use std::vec;

use crate::{
    chess::{
        self,
        color::Color,
        square::{Square, SQUARE_ITER},
    },
    utils,
};

use super::constants::{
    BISHOP_MAGIC_NUMBERS, BISHOP_RELEVANT_BITS, ROOK_MAGIC_NUMBERS, ROOK_RELEVANT_BITS,
};

fn mask_pawn_attacks(square: Square, side: Color) -> u64 {
    let mut bitboard: u64 = 0;
    utils::set_bit(&mut bitboard, u8::from(square));

    let mut attacks: u64 = 0;

    if side == Color::White {
        if (bitboard >> 7) & !(*chess::constants::FILE_A) != 0 {
            attacks |= bitboard >> 7;
        }
        if (bitboard >> 9) & !(*chess::constants::FILE_H) != 0 {
            attacks |= bitboard >> 9;
        }
    }

    if side == Color::Black {
        if (bitboard << 7) & !(*chess::constants::FILE_H) != 0 {
            attacks |= bitboard << 7;
        }
        if (bitboard << 9) & !(*chess::constants::FILE_A) != 0 {
            attacks |= bitboard << 9;
        }
    }

    return attacks;
}

fn mask_knight_attacks(square: Square) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    utils::set_bit(&mut bitboard, u8::from(square));

    if (bitboard >> 17) & !(*chess::constants::FILE_H) != 0 {
        attacks |= bitboard >> 17;
    }
    if (bitboard >> 15) & !(*chess::constants::FILE_A) != 0 {
        attacks |= bitboard >> 15;
    }
    if (bitboard >> 10) & !(*chess::constants::FILE_GH) != 0 {
        attacks |= bitboard >> 10;
    }
    if (bitboard >> 6) & !(*chess::constants::FILE_AB) != 0 {
        attacks |= bitboard >> 6;
    }
    if (bitboard << 17) & !(*chess::constants::FILE_A) != 0 {
        attacks |= bitboard << 17;
    }
    if (bitboard << 15) & !(*chess::constants::FILE_H) != 0 {
        attacks |= bitboard << 15;
    }
    if (bitboard << 10) & !(*chess::constants::FILE_AB) != 0 {
        attacks |= bitboard << 10;
    }
    if (bitboard << 6) & !(*chess::constants::FILE_GH) != 0 {
        attacks |= bitboard << 6;
    }

    return attacks;
}

fn mask_king_attacks(square: Square) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    utils::set_bit(&mut bitboard, u8::from(square));

    if (bitboard >> 9) & !(*chess::constants::FILE_H) != 0 {
        attacks |= bitboard >> 9;
    }
    if (bitboard >> 8) != 0 {
        attacks |= bitboard >> 8;
    }
    if (bitboard >> 7) & !(*chess::constants::FILE_A) != 0 {
        attacks |= bitboard >> 7;
    }
    if (bitboard >> 1) & !(*chess::constants::FILE_H) != 0 {
        attacks |= bitboard >> 1;
    }

    if (bitboard << 9) & !(*chess::constants::FILE_A) != 0 {
        attacks |= bitboard << 9;
    }
    if (bitboard << 8) != 0 {
        attacks |= bitboard << 8;
    }
    if (bitboard << 7) & !(*chess::constants::FILE_H) != 0 {
        attacks |= bitboard << 7;
    }
    if (bitboard << 1) & !(*chess::constants::FILE_A) != 0 {
        attacks |= bitboard << 1;
    }

    return attacks;
}

fn mask_bishop_attacks(square: Square) -> u64 {
    let mut attacks: u64 = 0;

    let square_rank = i8::from(square) / 8;
    let square_file = i8::from(square) % 8;

    for (rank, file) in ((square_rank + 1)..=6).zip((square_file + 1)..=6) {
        utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());
    }

    for (rank, file) in (1..=(square_rank - 1))
        .rev()
        .zip((1..=(square_file - 1)).rev())
    {
        utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());
    }

    for (rank, file) in ((square_rank + 1)..=6).zip((1..=(square_file - 1)).rev()) {
        utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());
    }

    for (rank, file) in (1..=(square_rank - 1)).rev().zip((square_file + 1)..=6) {
        utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());
    }

    return attacks;
}

fn mask_rook_attacks(square: Square) -> u64 {
    let mut attacks: u64 = 0;

    let square_rank = i8::from(square) / 8;
    let square_file = i8::from(square) % 8;

    for rank in (square_rank + 1)..=6 {
        utils::set_bit(&mut attacks, (rank * 8 + square_file).try_into().unwrap());
    }

    for rank in 1..=(square_rank - 1) {
        utils::set_bit(&mut attacks, (rank * 8 + square_file).try_into().unwrap());
    }

    for file in (square_file + 1)..=6 {
        utils::set_bit(&mut attacks, (square_rank * 8 + file).try_into().unwrap());
    }

    for file in 1..=(square_file - 1) {
        utils::set_bit(&mut attacks, (square_rank * 8 + file).try_into().unwrap());
    }

    return attacks;
}

fn generate_bishop_attacks_on_the_fly(square: Square, blockers: u64) -> u64 {
    let mut attacks: u64 = 0;

    let square_rank = i8::from(square) / 8;
    let square_file = i8::from(square) % 8;

    for (rank, file) in ((square_rank + 1)..=7).zip((square_file + 1)..=7) {
        utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());

        if ((1u64 << (rank * 8 + file)) & blockers) != 0 {
            break;
        }
    }

    for (rank, file) in (0..=(square_rank - 1))
        .rev()
        .zip((0..=(square_file - 1)).rev())
    {
        utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());

        if ((1u64 << (rank * 8 + file)) & blockers) != 0 {
            break;
        }
    }

    for (rank, file) in ((square_rank + 1)..=7).zip((0..=(square_file - 1)).rev()) {
        utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());

        if ((1u64 << (rank * 8 + file)) & blockers) != 0 {
            break;
        }
    }

    for (rank, file) in (0..=(square_rank - 1)).rev().zip((square_file + 1)..=7) {
        utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());

        if ((1u64 << (rank * 8 + file)) & blockers) != 0 {
            break;
        }
    }

    return attacks;
}

fn generate_rook_attacks_on_the_fly(square: Square, blockers: u64) -> u64 {
    let mut attacks: u64 = 0;

    let square_rank = i8::from(square) / 8;
    let square_file = i8::from(square) % 8;

    for rank in (square_rank + 1)..=7 {
        utils::set_bit(&mut attacks, (rank * 8 + square_file).try_into().unwrap());

        if ((1u64 << (rank * 8 + square_file)) & blockers) != 0 {
            break;
        }
    }

    for rank in (0..=(square_rank - 1)).rev() {
        utils::set_bit(&mut attacks, (rank * 8 + square_file).try_into().unwrap());

        if ((1u64 << (rank * 8 + square_file)) & blockers) != 0 {
            break;
        }
    }

    for file in (square_file + 1)..=7 {
        utils::set_bit(&mut attacks, (square_rank * 8 + file).try_into().unwrap());

        if ((1u64 << (square_rank * 8 + file)) & blockers) != 0 {
            break;
        }
    }

    for file in (0..=(square_file - 1)).rev() {
        utils::set_bit(&mut attacks, (square_rank * 8 + file).try_into().unwrap());

        if ((1u64 << (square_rank * 8 + file)) & blockers) != 0 {
            break;
        }
    }

    return attacks;
}

fn set_occupancy(index: u32, bits_in_mask: u64, attack_mask: u64) -> u64 {
    let mut occupancy = 0u64;
    let mut mutable_attack_mask = attack_mask;

    for i in 0..bits_in_mask {
        let square = utils::get_lsb(mutable_attack_mask);
        utils::clear_bit(&mut mutable_attack_mask, square);

        if (index & (1 << i)) != 0 {
            utils::set_bit(&mut occupancy, square);
        }
    }

    return occupancy;
}

fn initialize_slider_magic_attacks(is_bishop: bool) -> (Vec<Vec<u64>>, Vec<Vec<u64>>) {
    let mut magic_bishop_attacks = vec![vec![0; 512]; 64];
    let mut magic_rook_attacks = vec![vec![0; 4096]; 64];

    for square in SQUARE_ITER {
        let attack_mask = if is_bishop {
            mask_bishop_attacks(square)
        } else {
            mask_rook_attacks(square)
        };

        let relevant_bits_count = utils::count_bits(attack_mask);
        let occupancy_indices = 1 << relevant_bits_count;

        for index in 0..occupancy_indices {
            if is_bishop {
                let occupancy =
                    set_occupancy(index.try_into().unwrap(), relevant_bits_count, attack_mask);
                let magic_index = ((occupancy
                    .wrapping_mul(BISHOP_MAGIC_NUMBERS[usize::from(u8::from(square))]))
                    >> (64 - BISHOP_RELEVANT_BITS[usize::from(u8::from(square))]))
                    as usize;

                magic_bishop_attacks[usize::from(u8::from(square))][magic_index] =
                    generate_bishop_attacks_on_the_fly(square, occupancy);
            } else {
                let occupancy =
                    set_occupancy(index.try_into().unwrap(), relevant_bits_count, attack_mask);
                let magic_index = ((occupancy
                    .wrapping_mul(ROOK_MAGIC_NUMBERS[usize::from(u8::from(square))]))
                    >> (64 - ROOK_RELEVANT_BITS[usize::from(u8::from(square))]))
                    as usize;

                magic_rook_attacks[usize::from(u8::from(square))][magic_index] =
                    generate_rook_attacks_on_the_fly(square, occupancy);
            }
        }
    }

    return (magic_bishop_attacks, magic_rook_attacks);
}

pub fn set_file_rank_mask(file_num: i32, rank_num: i32) -> u64 {
    let mut mask: u64 = 0;

    for rank in 0..8 {
        for file in 0..8 {
            let square = rank * 8 + file;

            if file_num != -1 {
                if file == file_num {
                    utils::set_bit(&mut mask, square as u8);
                }
            } else if rank_num != -1 {
                if rank == rank_num {
                    utils::set_bit(&mut mask, square as u8);
                }
            }
        }
    }

    return mask;
}

lazy_static! {
    pub static ref PAWN_ATTACKS: [[u64; 64]; 2] = {
        let mut pawn_attacks = [[0; 64]; 2];
        for i in 0..64 {
            pawn_attacks[Color::White as usize][i] =
                mask_pawn_attacks(Square::from(i), Color::White);
            pawn_attacks[Color::Black as usize][i] =
                mask_pawn_attacks(Square::from(i), Color::Black);
        }
        return pawn_attacks;
    };
    pub static ref KNIGHT_ATTACKS: [u64; 64] = {
        let mut knight_attacks = [0; 64];
        for i in 0..64 {
            knight_attacks[i] = mask_knight_attacks(Square::from(i));
        }
        return knight_attacks;
    };
    pub static ref KING_ATTACKS: [u64; 64] = {
        let mut king_attacks = [0; 64];
        for i in 0..64 {
            king_attacks[i] = mask_king_attacks(Square::from(i));
        }
        return king_attacks;
    };
    pub static ref MAGIC_BISHOP_MASKS: [u64; 64] = {
        let mut magic_bishop_masks = [0; 64];
        for i in 0..64 {
            magic_bishop_masks[i] = mask_bishop_attacks(Square::from(i));
        }
        return magic_bishop_masks;
    };
    pub static ref MAGIC_ROOK_MASKS: [u64; 64] = {
        let mut magic_rook_masks = [0; 64];
        for i in 0..64 {
            magic_rook_masks[i] = mask_rook_attacks(Square::from(i));
        }
        return magic_rook_masks;
    };
    pub static ref MAGIC_BISHOP_ATTACKS: Vec<Vec<u64>> = {
        let (bishop_attacks, _) = initialize_slider_magic_attacks(true);
        return bishop_attacks;
    };
    pub static ref MAGIC_ROOK_ATTACKS: Vec<Vec<u64>> = {
        let (_, rook_attacks) = initialize_slider_magic_attacks(false);
        return rook_attacks;
    };
    pub static ref FILE_MASKS: [u64; 64] = {
        let mut file_masks = [0; 64];
        for rank in 0..8 {
            for file in 0..8 {
                let square = rank * 8 + file;
                file_masks[square] = set_file_rank_mask(file as i32, -1);
            }
        }
        return file_masks;
    };
    pub static ref RANK_MASKS: [u64; 64] = {
        let mut rank_masks = [0; 64];
        for rank in 0..8 {
            for file in 0..8 {
                let square = rank * 8 + file;
                rank_masks[square] = set_file_rank_mask(-1, rank as i32);
            }
        }
        return rank_masks;
    };
    pub static ref ISOLATED_PAWN_MASKS: [u64; 64] = {
        let mut isolated_pawn_masks = [0; 64];
        for rank in 0..8 {
            for file in 0..8 {
                let square = rank * 8 + file;
                isolated_pawn_masks[square] = set_file_rank_mask(file as i32 - 1, -1)
                    | set_file_rank_mask(file as i32 + 1, -1);
            }
        }
        return isolated_pawn_masks;
    };
    pub static ref WHITE_PASSED_PAWN_MASKS: [u64; 64] = {
        let mut white_passed_pawn_masks = [0; 64];
        for rank in 0..8 {
            for file in 0..8 {
                let square = rank * 8 + file;

                let m = set_file_rank_mask(file as i32 - 1, -1)
                    | set_file_rank_mask(file as i32 + 1, -1)
                    | set_file_rank_mask(file as i32, -1);

                white_passed_pawn_masks[square] = m;

                for i in 0..(8 - rank) {
                    white_passed_pawn_masks[square] &= !RANK_MASKS[(7 - i) * 8 + file];
                }
            }
        }
        return white_passed_pawn_masks;
    };
    pub static ref BLACK_PASSED_PAWN_MASKS: [u64; 64] = {
        let mut black_passed_pawn_masks = [0; 64];
        for rank in 0..8 {
            for file in 0..8 {
                let square = rank * 8 + file;

                let m = set_file_rank_mask(file as i32 - 1, -1)
                    | set_file_rank_mask(file as i32 + 1, -1)
                    | set_file_rank_mask(file as i32, -1);

                black_passed_pawn_masks[square] = m;

                for i in 0..(rank + 1) {
                    black_passed_pawn_masks[square] &= !RANK_MASKS[i * 8 + file];
                }
            }
        }
        return black_passed_pawn_masks;
    };
}
