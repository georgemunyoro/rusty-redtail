#[inline(always)]
pub fn get_bit(bitboard: u64, bit_index: u8) -> u64 {
    return (bitboard) & (1u64 << bit_index);
}

#[inline(always)]
pub fn set_bit(bitboard: &mut u64, bit_index: u8) {
    *bitboard |= 1u64 << bit_index;
}

#[inline(always)]
pub fn count_bits(bitboard: u64) -> u64 {
    let mut count = 0;
    let mut temp_bitboard = bitboard;
    while temp_bitboard != 0 {
        temp_bitboard &= temp_bitboard - 1;
        count += 1;
    }
    return count;
}

#[inline(always)]
pub fn get_lsb(bitboard: u64) -> u8 {
    return bitboard.trailing_zeros() as u8;
}

#[inline(always)]
pub fn pop_lsb(bitboard: &mut u64) -> u8 {
    let lsb = get_lsb(*bitboard);
    *bitboard &= *bitboard - 1;
    return lsb;
}

#[inline(always)]
pub fn clear_bit(bitboard: &mut u64, bit_index: u8) {
    *bitboard &= !(1u64 << bit_index);
}

pub fn _print_bitboard(bitboard: u64) {
    for i in 0..64 {
        if i % 8 == 0 {
            println!();
        }
        print!(
            "{} ",
            match get_bit(bitboard, i) {
                0 => "•",
                _ => "\x1b[31m*\x1b[0m",
            }
        );
    }
    println!()
}

pub fn get_pseudorandom_number_u32(state: &mut u32) -> u32 {
    let mut number = *state;
    number ^= number << 13;
    number ^= number >> 17;
    number ^= number << 5;
    *state = number;
    return number;
}

pub fn get_pseudorandom_number_u64(state: &mut u32) -> u64 {
    let n1 = (get_pseudorandom_number_u32(state) as u64) & 0xFFFF;
    let n2 = (get_pseudorandom_number_u32(state) as u64) & 0xFFFF;
    let n3 = (get_pseudorandom_number_u32(state) as u64) & 0xFFFF;
    let n4 = (get_pseudorandom_number_u32(state) as u64) & 0xFFFF;

    return n1 | (n2 << 16) | (n3 << 32) | (n4 << 48);
}

pub fn _print_progress_bar(progress: f32, bar_length: usize) {
    let filled_length = (progress * bar_length as f32) as usize;
    let empty_length = bar_length - filled_length;

    print!("\n[");
    for _ in 0..filled_length {
        print!("=");
    }
    for _ in 0..empty_length {
        print!(" ");
    }
    print!("] {:.2}%\n", progress * 100.0);
}
