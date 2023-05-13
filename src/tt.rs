use crate::{
    board::{Board, Position},
    chess,
    movegen::MoveGenerator,
};

const HASH_SIZE: usize = 0x400000;

#[derive(PartialEq, Eq, Clone, Copy)]
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
    pub m: Option<chess::Move>,
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
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        return TranspositionTable {
            table: vec![TranspositionTableEntry::new(); HASH_SIZE],
        };
    }

    /// Stores a new entry in the transposition table. If the entry already exists, it is overwritten.
    pub fn save(
        &mut self,
        key: u64,
        depth: u8,
        flag: TranspositionTableEntryFlag,
        value: i32,
        m: Option<chess::Move>,
    ) {
        self.table[(key as usize % HASH_SIZE)] = TranspositionTableEntry {
            key,
            depth,
            flag,
            value,
            m,
        }
    }

    /// Returns the entry if it exists, otherwise returns None.
    pub fn probe(&self, key: u64, depth: u8, alpha: i32, beta: i32) -> Option<i32> {
        let entry = self.table[key as usize % HASH_SIZE];

        if entry.key == key {
            if entry.depth >= depth {
                if entry.flag == TranspositionTableEntryFlag::EXACT {
                    return Some(entry.value);
                }
                if entry.flag == TranspositionTableEntryFlag::ALPHA && entry.value <= alpha {
                    return Some(alpha);
                }
                if entry.flag == TranspositionTableEntryFlag::BETA && entry.value >= beta {
                    return Some(beta);
                }
            }
        }

        return None;
    }

    pub fn clear(&mut self) {
        self.table = vec![TranspositionTableEntry::new(); HASH_SIZE];
    }

    pub fn get_pv_line(&self, position: &mut Position) -> Vec<TranspositionTableEntry> {
        let mut pv_line = Vec::new();

        loop {
            let entry = self.table[position.hash as usize % HASH_SIZE];

            if entry.key != position.hash || entry.m.is_none() {
                break;
            }

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
