use crate::chess::square::Square;

pub trait Bitboard {
    fn get_bit(&self, bit_index: u8) -> u64;
    fn set_bit(&mut self, bit_index: Square);
    fn get_lsb(&self) -> u8;
    fn pop_lsb(&mut self) -> u8;
    fn clear_bit(&mut self, bit_index: Square);
    fn test_bit(&self, bit_index: u8) -> bool {
        return self.get_bit(bit_index) != 0;
    }
}

impl Bitboard for u64 {
    fn get_bit(&self, bit_index: u8) -> u64 {
        return (self) & (1u64 << bit_index);
    }

    fn set_bit(&mut self, bit_index: Square) {
        *self |= 1u64 << bit_index as u8;
    }

    fn get_lsb(&self) -> u8 {
        return self.trailing_zeros() as u8;
    }

    fn pop_lsb(&mut self) -> u8 {
        let lsb = self.get_lsb();
        *self &= *self - 1;
        return lsb;
    }

    fn clear_bit(&mut self, bit_index: Square) {
        *self &= !(1u64 << bit_index as u8);
    }
}
