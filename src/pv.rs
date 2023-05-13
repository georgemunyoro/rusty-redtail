use crate::{
    board::{self, Board},
    chess,
    movegen::MoveGenerator,
};

pub const PV_SIZE: usize = 0x400000;
const MAX_PV_SIZE: usize = 64;

#[derive(Clone, Copy, Debug)]
pub struct PrincipalVariationEntry {
    key: u64,
    m: chess::Move,
}

impl PrincipalVariationEntry {
    pub fn default() -> PrincipalVariationEntry {
        PrincipalVariationEntry {
            key: 0,
            m: chess::NULL_MOVE,
        }
    }
}

pub struct PrincipalVariationTable {
    pub table: Vec<PrincipalVariationEntry>,
}

impl PrincipalVariationTable {
    pub fn new() -> PrincipalVariationTable {
        println!("pv {} entries", PV_SIZE);
        return PrincipalVariationTable {
            table: vec![PrincipalVariationEntry::default(); PV_SIZE as usize],
        };
    }

    pub fn clear(&mut self) {
        self.table.fill(PrincipalVariationEntry::default());
    }

    pub fn store(&mut self, key: u64, m: chess::Move) {
        let index = key % PV_SIZE as u64;
        self.table[index as usize] = PrincipalVariationEntry { key, m }
    }

    pub fn probe(&self, key: u64) -> Option<chess::Move> {
        let index = key % PV_SIZE as u64;
        let entry = self.table[index as usize];
        if entry.key == key {
            return Some(entry.m);
        }
        return None;
    }

    pub fn get_pv_line(&self, position: &mut board::Position) -> Vec<chess::Move> {
        let mut pv_line = Vec::with_capacity(MAX_PV_SIZE);

        loop {
            let move_found = self.probe(position.hash);
            let legal_moves = position.generate_legal_moves();

            match move_found {
                None => {
                    break;
                }
                Some(m) => {
                    if !legal_moves.contains(&m) {
                        break;
                    }

                    pv_line.push(m);
                    position.make_move(m, false);
                }
            }
        }

        for _ in 0..pv_line.len() {
            position.unmake_move();
        }

        return pv_line;
    }
}
