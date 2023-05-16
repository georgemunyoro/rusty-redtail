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

#[derive(Clone, Copy)]
pub struct TranspositionTableEntry {
    pub key: u64,
    pub depth: u8,
    pub flag: TranspositionTableEntryFlag,
    pub value: i32,
    pub m: Option<skaak::_move::BitPackedMove>,
}

impl TranspositionTableEntry {
    pub fn new() -> TranspositionTableEntry {
        return TranspositionTableEntry {
            depth: 0,
            flag: TranspositionTableEntryFlag::NULL,
            key: 0,
            value: 0,
            m: None,
        };
    }
}

pub struct TranspositionTable {
    table: Vec<TranspositionTableEntry>,
    size: u64,
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        return TranspositionTable {
            table: vec![TranspositionTableEntry::new(); HASH_SIZE],
            size: 0,
        };
    }

    /// Stores a new entry in the transposition table. If the entry already exists, it is overwritten.
    pub fn save(
        &mut self,
        key: u64,
        depth: u8,
        flag: TranspositionTableEntryFlag,
        value: i32,
        m: Option<skaak::_move::BitPackedMove>,
    ) {
        let hash_index = key as usize & (HASH_SIZE - 1);

        if self.table[hash_index].key == 0 {
            self.size += 1;
        }

        self.table[hash_index] = TranspositionTableEntry {
            key,
            depth,
            flag,
            value,
            m,
        }
    }

    /// Returns the entry if it exists, otherwise returns None.
    pub fn get(&self, key: u64) -> Option<TranspositionTableEntry> {
        let entry: TranspositionTableEntry = self.table[key as usize & (HASH_SIZE - 1)];
        if entry.key == key && entry.flag == TranspositionTableEntryFlag::EXACT {
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

        if entry.key == key {
            if entry.depth >= depth {
                if entry.flag == TranspositionTableEntryFlag::EXACT {
                    return Some((entry.value, entry.flag));
                }
                if entry.flag == TranspositionTableEntryFlag::ALPHA && entry.value <= alpha {
                    return Some((alpha, entry.flag));
                }
                if entry.flag == TranspositionTableEntryFlag::BETA && entry.value >= beta {
                    return Some((beta, entry.flag));
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

            if entry.key != position.hash
                || entry.m.is_none()
                || pv_line.len() > 64
                || positions_encountered.contains(&position.hash)
            {
                break;
            }

            positions_encountered.push(position.hash);
            pv_line.push(entry);
            position.make_move(entry.m.unwrap(), false);

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
