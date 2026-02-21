use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

use crate::{
    board::{Board, Position},
    chess,
    movegen::MoveGenerator,
};

/*
   TranspositionTable
   ------------------
   A transposition table is a hash table that stores information about positions that have already been searched.
   This allows the engine to avoid searching the same position multiple times, and can also be used to detect
   repetitions.

   The table is lock-free and thread-safe: each entry uses AtomicU64 for key/data and AtomicU8 for age.
   Torn reads from concurrent access are detected by the XOR key verification (key == hash ^ data).
*/

/// Internal atomic storage for a single TT slot.
struct AtomicTTEntry {
    key: AtomicU64,
    data: AtomicU64,
    age: AtomicU8,
}

pub struct TranspositionTable {
    table: Vec<AtomicTTEntry>,
    size: AtomicU64,
    age: AtomicU8,
    pub hash_size: usize,
}

// Safety: All fields use atomic types or are immutable after construction.
unsafe impl Sync for TranspositionTable {}

impl TranspositionTable {
    pub fn new(hash_size_in_mb: usize) -> TranspositionTable {
        let hash_size =
            hash_size_in_mb * 1024 * 1024 / std::mem::size_of::<TranspositionTableEntry>();
        let mut table = Vec::with_capacity(hash_size);
        for _ in 0..hash_size {
            table.push(AtomicTTEntry {
                key: AtomicU64::new(0),
                data: AtomicU64::new(0),
                age: AtomicU8::new(0),
            });
        }
        TranspositionTable {
            table,
            size: AtomicU64::new(0),
            age: AtomicU8::new(0),
            hash_size,
        }
    }

    pub fn increment_age(&self) {
        self.age.fetch_add(1, Ordering::Relaxed);
    }

    /// Clears the transposition table
    pub fn clear(&self) {
        for entry in self.table.iter() {
            entry.key.store(0, Ordering::Relaxed);
            entry.data.store(0, Ordering::Relaxed);
            entry.age.store(0, Ordering::Relaxed);
        }
        self.size.store(0, Ordering::Relaxed);
        self.age.store(0, Ordering::Relaxed);
    }

    /// Loads an entry from the table as a plain value type.
    fn load_entry(&self, index: usize) -> TranspositionTableEntry {
        let slot = &self.table[index];
        TranspositionTableEntry {
            key: slot.key.load(Ordering::Relaxed),
            data: slot.data.load(Ordering::Relaxed),
            age: slot.age.load(Ordering::Relaxed),
        }
    }

    /// Stores a new entry in the transposition table. If the entry already exists, it is overwritten.
    pub fn save(
        &self,
        key: u64,
        depth: u8,
        flag: TranspositionTableEntryFlag,
        value: i32,
        m: chess::_move::BitPackedMove,
    ) {
        let hash_index = key as usize & (self.hash_size - 1);

        /*
                      score (25)        | flag (2) |  depth (8) |            move (30)
           00000000 00000000 00000000 0 |    00    |  00000000  | 00000000 00000000 0000000 00000
        */
        let data = ((value + 50_000) as u64)
            | ((flag as u64) << 25)
            | ((depth as u64) << 27)
            | ((m.move_bits as u64) << 35);

        let existing = self.load_entry(hash_index);
        let current_age = self.age.load(Ordering::Relaxed);

        let replace = if existing.key == 0 {
            self.size.fetch_add(1, Ordering::Relaxed);
            true
        } else {
            existing.age < current_age || existing.get_depth() <= depth
        };

        if !replace {
            return;
        }

        let slot = &self.table[hash_index];
        slot.data.store(data, Ordering::Relaxed);
        slot.key.store(key ^ data, Ordering::Relaxed);
        slot.age.store(current_age, Ordering::Relaxed);
    }

    /// Returns the entry if the hash matches (any bound type), otherwise returns None.
    pub fn get(&self, key: u64) -> Option<TranspositionTableEntry> {
        let entry = self.load_entry(key as usize & (self.hash_size - 1));
        if entry.key == (key ^ entry.data)
            && entry.get_flag() != TranspositionTableEntryFlag::NULL
        {
            return Some(entry);
        }
        None
    }

    /// Returns the entry if it exists and is suitable for the given lower and upper bound, otherwise returns None.
    pub fn probe_entry(
        &self,
        key: u64,
        depth: u8,
        alpha: i32,
        beta: i32,
    ) -> TranspositionTableEntry {
        let entry = self.load_entry(key as usize & (self.hash_size - 1));

        if entry.key == (key ^ entry.data) {
            if entry.get_depth() >= depth {
                if entry.get_flag() == TranspositionTableEntryFlag::EXACT {
                    return entry;
                }
                if entry.get_flag() == TranspositionTableEntryFlag::ALPHA
                    && entry.get_value() <= alpha
                {
                    return entry;
                }
                if entry.get_flag() == TranspositionTableEntryFlag::BETA
                    && entry.get_value() >= beta
                {
                    return entry;
                }
            }
        }

        TranspositionTableEntry::new()
    }

    /// Returns how full the table is, as a number between 0 and 1000.
    pub fn get_hashfull(&self) -> u32 {
        (self.size.load(Ordering::Relaxed) as f32 / self.hash_size as f32 * 1000.0) as u32
    }

    /// Returns the principal variation line for the given position.
    pub fn get_pv_line(&self, position: &mut Position) -> Vec<TranspositionTableEntry> {
        let mut pv_line: Vec<TranspositionTableEntry> = Vec::new();

        loop {
            let entry = self.load_entry(position.hash as usize & (self.hash_size - 1));

            if entry.key != (position.hash ^ entry.data)
                || entry.get_move() == chess::_move::BitPackedMove::default()
                || pv_line.len() > 64
            {
                break;
            }

            pv_line.push(entry);
            position.make_move(entry.get_move(), false);

            if position.is_in_check() && position.generate_legal_moves().len() == 0 {
                break;
            }
        }

        for _ in 0..pv_line.len() {
            position.unmake_move();
        }

        pv_line
    }
}

#[derive(Clone, Copy)]
pub struct TranspositionTableEntry {
    key: u64,
    data: u64,
    pub age: u8,
}

impl TranspositionTableEntry {
    pub fn new() -> TranspositionTableEntry {
        TranspositionTableEntry {
            key: 0,
            data: 0,
            age: 0,
        }
    }
}

impl TranspositionTableEntry {
    /// Returns the depth of the search that produced the score.
    pub fn get_depth(&self) -> u8 {
        (self.data >> 27) as u8 & 0xFF
    }

    /// Returns the type of score (exact, alpha, beta, or null).
    pub fn get_flag(&self) -> TranspositionTableEntryFlag {
        TranspositionTableEntryFlag::from_u8((self.data >> 25) as u8 & 0x3)
    }

    /// Returns whether the entry is valid.
    pub fn is_valid(&self) -> bool {
        self.key != 0
    }

    /// Returns the evaluation of the position.
    pub fn get_value(&self) -> i32 {
        ((self.data as i32 & 0x1FFFFFF) - 50_000) as i32
    }

    /// Returns the move that produced the score.
    pub fn get_move(&self) -> chess::_move::BitPackedMove {
        chess::_move::BitPackedMove {
            move_bits: (self.data >> 35) as u32,
        }
    }
}

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
