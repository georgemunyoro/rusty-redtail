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
*/
pub struct TranspositionTable {
    /// The actual hash table.
    table: Vec<TranspositionTableEntry>,

    /// The number of entries in the table.
    size: u64,

    /// The age of the table. This is used to determine which entries to replace.
    pub age: u8,

    /// The size of the table.
    pub hash_size: usize,
}

impl TranspositionTable {
    pub fn new(hash_size_in_mb: usize) -> TranspositionTable {
        let hash_size =
            hash_size_in_mb * 1024 * 1024 / std::mem::size_of::<TranspositionTableEntry>();
        return TranspositionTable {
            table: vec![TranspositionTableEntry::new(); hash_size],
            size: 0,
            age: 0,
            hash_size,
        };
    }

    /// Stores a new entry in the transposition table. If the entry already exists, it is overwritten.
    pub fn save(
        &mut self,
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

        let mut replace = false;

        if self.table[hash_index].key == 0 {
            self.size += 1;
            replace = true;
        } else {
            if self.table[hash_index].age < self.age || self.table[hash_index].get_depth() <= depth
            {
                replace = true;
            }
        }

        if !replace {
            return;
        }

        self.table[hash_index] = TranspositionTableEntry {
            key: key ^ data,
            data,
            age: self.age,
        }
    }

    /// Returns the entry if it exists, otherwise returns None.
    pub fn get(&self, key: u64) -> Option<TranspositionTableEntry> {
        let entry: TranspositionTableEntry = self.table[key as usize & (self.hash_size - 1)];
        if entry.key == (key ^ entry.data) && entry.get_flag() == TranspositionTableEntryFlag::EXACT
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
        let entry: TranspositionTableEntry = self.table[key as usize & (self.hash_size - 1)];

        if entry.key == (key ^ entry.data) {
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

    /// Returns how full the table is, as a number between 0 and 1000.
    pub fn get_hashfull(&self) -> u32 {
        return (self.size as f32 / self.hash_size as f32 * 1000.0) as u32;
    }

    /// Returns the principal variation line for the given position.
    pub fn get_pv_line(&self, position: &mut Position) -> Vec<TranspositionTableEntry> {
        let mut pv_line: Vec<TranspositionTableEntry> = Vec::new();
        let mut positions_encountered: Vec<u64> = Vec::new();

        loop {
            let entry: TranspositionTableEntry =
                self.table[position.hash as usize & (self.hash_size - 1)];

            if entry.key != (position.hash ^ entry.data)
                || entry.get_move() == chess::_move::BitPackedMove::default()
                || pv_line.len() > 64
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

#[derive(Clone, Copy)]
pub struct TranspositionTableEntry {
    key: u64,
    data: u64,
    pub age: u8,
}

/*
   TranspositionTableEntry
   -----------------------
   A transposition table entry is a single entry in the transposition table. It contains the following information:
   - The key, which is the hash of the position XORed with the data.
   - The data, which contains the following information:
     - The score, which is the evaluation of the position.
     - The flag, which is the type of score (exact, alpha, beta, or null).
     - The depth, which is the depth of the search that produced the score.
     - The move, which is the move that produced the score.
   - The age, which is used to determine which entries to replace.

   When retrieving an entry from the transposition table, the key is XORed with the data to ensure that the entry
    is valid.
*/
impl TranspositionTableEntry {
    pub fn new() -> TranspositionTableEntry {
        return TranspositionTableEntry {
            key: 0,
            data: 0,
            age: 0,
        };
    }
}

impl TranspositionTableEntry {
    /// Returns the depth of the search that produced the score.
    pub fn get_depth(&self) -> u8 {
        return (self.data >> 27) as u8 & 0xF;
    }

    /// Returns the type of score (exact, alpha, beta, or null).
    pub fn get_flag(&self) -> TranspositionTableEntryFlag {
        return TranspositionTableEntryFlag::from_u8((self.data >> 25) as u8 & 0x3);
    }

    /// Returns the evaluation of the position.
    pub fn get_value(&self) -> i32 {
        return ((self.data as i32 & 0x1FFFFFF) - 50_000) as i32;
    }

    /// Returns the move that produced the score.
    pub fn get_move(&self) -> chess::_move::BitPackedMove {
        return chess::_move::BitPackedMove {
            move_bits: (self.data >> 35) as u32,
        };
    }
}

/*
   The TranspositionTableEntryFlag enum is used to indicate the type of score that is stored in a transposition table entry.
   - EXACT: The score is an exact score.
   - BETA: The score is a lower bound.
   - ALPHA: The score is an upper bound.
   - NULL: The score is a null score, this indicates an invalid entry.
*/
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
