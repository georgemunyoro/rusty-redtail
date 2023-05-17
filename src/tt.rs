use crate::{
    board::{Board, Position},
    movegen::MoveGenerator,
    skaak,
};

const HASH_SIZE: usize = 0x1000000;

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TranspositionTableEntryFlag {
    EXACT,
    BETA,
    ALPHA,
    NULL,
}

impl TranspositionTableEntryFlag {
    pub fn from_u8(value: u8) -> TranspositionTableEntryFlag {
        match value {
            0 => TranspositionTableEntryFlag::EXACT,
            1 => TranspositionTableEntryFlag::BETA,
            2 => TranspositionTableEntryFlag::ALPHA,
            3 => TranspositionTableEntryFlag::NULL,
            _ => panic!("Invalid TranspositionTableEntryFlag"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TranspositionTableEntry {
    smp_key: u64,
    smp_data: u64,
    pub age: u8,
}

impl TranspositionTableEntry {
    pub fn get_depth(&self) -> u8 {
        return (self.smp_data >> 27) as u8 & 0xF;
    }

    pub fn get_flag(&self) -> TranspositionTableEntryFlag {
        return TranspositionTableEntryFlag::from_u8((self.smp_data >> 25) as u8 & 0x3);
    }

    pub fn get_value(&self) -> i32 {
        return ((self.smp_data as i32 & 0x1FFFFFF) - 50_000) as i32;
    }

    pub fn get_move(&self) -> skaak::_move::BitPackedMove {
        return skaak::_move::BitPackedMove {
            move_bits: (self.smp_data >> 35) as u32,
        };
    }
}

impl TranspositionTableEntry {
    pub fn new() -> TranspositionTableEntry {
        return TranspositionTableEntry {
            smp_key: 0,
            smp_data: 0,
            age: 0,
        };
    }
}

pub struct TranspositionTable {
    table: Vec<TranspositionTableEntry>,
    size: u64,
    pub age: u8,
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        return TranspositionTable {
            table: vec![TranspositionTableEntry::new(); HASH_SIZE],
            size: 0,
            age: 0,
        };
    }

    /// Stores a new entry in the transposition table. If the entry already exists, it is overwritten.
    pub fn save(
        &mut self,
        key: u64,
        depth: u8,
        flag: TranspositionTableEntryFlag,
        value: i32,
        m: skaak::_move::BitPackedMove,
    ) {
        let hash_index = key as usize & (HASH_SIZE - 1);

        /*
                      score (25)        | flag (2) |  depth (8) |            move (30)
           00000000 00000000 00000000 0 |    00    |  00000000  | 00000000 00000000 0000000 00000
        */
        let data = ((value + 50_000) as u64)
            | ((flag as u64) << 25)
            | ((depth as u64) << 27)
            | ((m.move_bits as u64) << 35);

        let mut replace = false;

        if self.table[hash_index].smp_key == 0 {
            self.size += 1;
            replace = true;
        } else {
            if self.table[hash_index].age < self.age || self.table[hash_index].get_depth() <= depth {
                replace = true;
            }
        }

        if !replace {
            return;
        }

        self.table[hash_index] = TranspositionTableEntry {
            smp_key: key ^ data,
            smp_data: data,
            age: self.age,
        }
    }

    /// Returns the entry if it exists, otherwise returns None.
    pub fn get(&self, key: u64) -> Option<TranspositionTableEntry> {
        let entry: TranspositionTableEntry = self.table[key as usize & (HASH_SIZE - 1)];
        if entry.smp_key == (key ^ entry.smp_data)
            && entry.get_flag() == TranspositionTableEntryFlag::EXACT
        {
            return Some(entry);
        }
        return None;
    }

    /// Returns the entry if it exists and is suitable for the given lower and upper bound, otherwise returns None.
    pub fn probe_entry(
        &self,
        key: u64,
        depth: u8,
        alpha: i32,
        beta: i32,
    ) -> Option<(i32, TranspositionTableEntryFlag)> {
        let entry: TranspositionTableEntry = self.table[key as usize & (HASH_SIZE - 1)];

        if entry.smp_key == (key ^ entry.smp_data) {
            if entry.get_depth() >= depth {
                if entry.get_flag() == TranspositionTableEntryFlag::EXACT {
                    return Some((entry.get_value(), entry.get_flag()));
                }
                if entry.get_flag() == TranspositionTableEntryFlag::ALPHA
                    && entry.get_value() <= alpha
                {
                    return Some((alpha, entry.get_flag()));
                }
                if entry.get_flag() == TranspositionTableEntryFlag::BETA
                    && entry.get_value() >= beta
                {
                    return Some((beta, entry.get_flag()));
                }
            }
        }

        return None;
    }

    pub fn get_hashfull(&self) -> u32 {
        return (self.size as f32 / HASH_SIZE as f32 * 1000.0) as u32;
    }

    pub fn get_pv_line(&self, position: &mut Position) -> Vec<TranspositionTableEntry> {
        let mut pv_line: Vec<TranspositionTableEntry> = Vec::new();
        let mut positions_encountered: Vec<u64> = Vec::new();

        loop {
            let entry: TranspositionTableEntry =
                self.table[position.hash as usize & (HASH_SIZE - 1)];

            if entry.smp_key != (position.hash ^ entry.smp_data)
                || entry.get_move() == skaak::_move::BitPackedMove::default()
                || pv_line.len() > 64
                || positions_encountered.contains(&position.hash)
            {
                break;
            }

            positions_encountered.push(position.hash);
            pv_line.push(entry);
            position.make_move(entry.get_move(), false);

            if position.is_in_check() && position.generate_legal_moves().len() == 0 {
                break;
            }
        }

        for _ in 0..pv_line.len() {
            position.unmake_move();
        }

        return pv_line;
    }
}
