use std::sync::{Arc, Mutex};

use board::Board;
use uci::UCI;

mod board;
mod chess;
mod movegen;
mod pst;
mod search;
mod tt;
mod uci;
mod utils;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Debug, Copy, Clone)]
pub struct Cutoffs {
    total: u32,
    move_1: u32,
    move_2: u32,
    avg_cutoff_move_no: f32,
}

impl Cutoffs {
    pub fn new() -> Self {
        Self {
            total: 0,
            move_1: 0,
            move_2: 0,
            avg_cutoff_move_no: 0.0,
        }
    }
}

fn main() {
    let mut position = <board::Position as board::Board>::new(None);

    let mut global_cutoffs = Cutoffs::new();

    for fen in X {
        position.set_fen(String::from(fen));
        let mut evaluator = search::evaluate::Evaluator::new();

        let mut search_options = search::options::SearchOptions::new();
        search_options.depth = Some(8);
        let is_searching = Arc::new(Mutex::new(true));
        let transposition_table = Arc::new(Mutex::new(tt::TranspositionTable::new(32)));

        evaluator.get_best_move(
            &mut position,
            search_options,
            is_searching,
            transposition_table,
            0,
            1,
        );

        println!("\n===========================");
        println!("Total recorded cutoffs: {}", evaluator.result.cutoffs.total);
        println!(
            "Cutoffs with 1st move: {}",
            (evaluator.result.cutoffs.move_1 as f32 / evaluator.result.cutoffs.total as f32)
                * 100.0
        );
        println!(
            "Cutoffs with 2nd move: {}",
            (evaluator.result.cutoffs.move_2 as f32 / evaluator.result.cutoffs.total as f32)
                * 100.0
        );
        println!(
            "Avg. move no that causes cut off: {}",
            evaluator.result.cutoffs.avg_cutoff_move_no as f32
                / evaluator.result.cutoffs.total as f32
        );
        println!("===========================\n");

        global_cutoffs.total += evaluator.result.cutoffs.total;
        global_cutoffs.avg_cutoff_move_no += evaluator.result.cutoffs.avg_cutoff_move_no;
        global_cutoffs.move_1 += evaluator.result.cutoffs.move_1;
        global_cutoffs.move_2 += evaluator.result.cutoffs.move_2;
    }

    println!("\n===========================");
    println!("\n======GLOBAL CUTOFFS=======");
    println!("\n===========================");
    println!("Total recorded cutoffs: {}", global_cutoffs.total);
    println!(
        "Cutoffs with 1st move: {}",
        (global_cutoffs.move_1 as f32 / global_cutoffs.total as f32) * 100.0
    );
    println!(
        "Cutoffs with 2nd move: {}",
        (global_cutoffs.move_2 as f32 / global_cutoffs.total as f32) * 100.0
    );
    println!(
        "Avg. move no that causes cut off: {}",
        global_cutoffs.avg_cutoff_move_no as f32 / global_cutoffs.total as f32
    );
    println!("===========================\n");
    println!("===========================\n");
}

const X: [&str; 300] = [
    "rnbqk2r/pp1p1ppp/4pn2/2b5/8/4P3/PPP1BPPP/RNBQK1NR w KQkq - 0 5",
    "r1bqk1nr/pppp1pbp/2n3p1/4p3/2P1P3/3P3P/PP3PP1/RNBQKBNR w KQkq - 1 5",
    "rnbqk1nr/pp1pbppp/2p5/3Np3/2P5/6P1/PP1PPP1P/R1BQKBNR w KQkq - 0 5",
    "rnbqk2r/pppp1pp1/5n1p/4p3/1bP5/3PP3/PP2BPPP/RNBQK1NR w KQkq - 1 5",
    "r1bqkbnr/p2p1ppp/2p1p3/2p5/4P3/2N5/PPPP1PPP/R1BQK1NR w KQkq - 0 5",
    "rnbqkb1r/pp3ppp/3ppn2/2p5/4P3/3P3P/PPP1BPP1/RNBQK1NR w KQkq - 1 5",
    "r1bqkb1r/pp1p1ppp/2n1pn2/2p5/2P5/P1N2N2/1P1PPPPP/R1BQKB1R w KQkq - 1 5",
    "rnbqkb1r/pp3ppp/2p1pn2/3p4/4P3/3P1N2/PPPN1PPP/R1BQKB1R w KQkq - 2 5",
    "r1bqkb1r/pppp1ppp/4pn2/8/3nP3/2N5/PPPPBPPP/R1BQK1NR w KQkq - 0 5",
    "r1b1kb1r/ppqppppp/2n2n2/2p5/4P3/2N2N1P/PPPP1PP1/R1BQKB1R w KQkq - 3 5",
    "rn1qkbnr/pbpp1p1p/1p2p1p1/8/2P5/1P2PN2/P2P1PPP/RNBQKB1R w KQkq - 1 5",
    "r1bqkb1r/pp1p1ppp/2n1pn2/2p5/4P3/N1P2N2/PP1P1PPP/R1BQKB1R w KQkq - 2 5",
    "rn1qkb1r/pbpp1ppp/1p2pn2/8/2P5/2N1P3/PP1PBPPP/R1BQK1NR w KQkq - 2 5",
    "rn1qkb1r/ppp1pppp/5nb1/3p4/8/PP3N2/1BPPPPPP/RN1QKB1R w KQkq - 1 5",
    "rnbqk1nr/1pp1bppp/p3p3/3p4/2P5/4P2P/PP1PBPP1/RNBQK1NR w KQkq - 0 5",
    "r1bqkb1r/pp2pppp/2n2n2/2pp4/2P5/3P2P1/PP2PPBP/RNBQK1NR w KQkq - 3 5",
    "rnbqk2r/1ppp1ppp/5n2/p2Np3/1bP5/1P6/P2PPPPP/R1BQKBNR w KQkq - 1 5",
    "rnb1kb1r/pp1ppppp/1qp5/7n/3P4/5NB1/PPP1PPPP/RN1QKB1R w KQkq - 6 5",
    "r1bqkbnr/pp1n1ppp/2pp4/4p3/3P4/2N3PP/PPP1PP2/R1BQKBNR w KQkq - 0 5",
    "rnbqkb1r/p1p2ppp/1p2pn2/3p4/3P4/P3PN2/1PP2PPP/RNBQKB1R w KQkq - 2 5",
    "rn1qkb1r/pp2pppp/2p2n2/3p1b2/8/P4NP1/1PPPPPBP/RNBQK2R w KQkq - 0 5",
    "r1bqkb1r/pp1p1ppp/2n2n2/2p1p3/2P1P3/2N2N2/PP1P1PPP/R1BQKB1R w KQkq - 0 5",
    "rnbqk2r/ppp1bppp/4pn2/3p4/2PP4/P7/1PQ1PPPP/RNB1KBNR w KQkq - 0 5",
    "rnbqkb1r/1ppp1p1p/4pnp1/p2P4/2P2B2/8/PP2PPPP/RN1QKBNR w KQkq - 0 5",
    "rnbqkb1r/pppp1p1p/5np1/8/2Pp4/P3P3/1P3PPP/RNBQKBNR w KQkq - 0 5",
    "r2qkbnr/pppn1ppp/4p3/3p1b2/3P1B2/P3P3/1PP2PPP/RN1QKBNR w KQkq - 1 5",
    "rnbqkb1r/2pp1ppp/pp2pn2/8/8/1P2P3/PBPPBPPP/RN1QK1NR w KQkq - 2 5",
    "rn1qkb1r/pp2pppp/2p2n2/3p1b2/8/P2P1NP1/1PP1PP1P/RNBQKB1R w KQkq - 1 5",
    "r1bqk2r/ppppnppp/2n5/2b1p3/4P3/2NP4/PPP1NPPP/R1BQKB1R w KQkq - 5 5",
    "rnbqkb1r/pp2pp1p/2p2np1/3p4/3P4/2P3P1/PP1NPP1P/R1BQKBNR w KQkq - 0 5",
    "rnbqk2r/p1ppbppp/4pn2/1p6/8/3P1NP1/PPPNPP1P/R1BQKB1R w KQkq - 2 5",
    "rnbqkb1r/pp2ppp1/5n1p/2pp4/3P4/4P2P/PPP1BPP1/RNBQK1NR w KQkq - 0 5",
    "rn1qk1nr/ppp2ppp/3p4/2b1p3/4P1b1/2N2NP1/PPPP1P1P/R1BQKB1R w KQkq - 2 5",
    "rn1qkb1r/pp2pppp/2p2n2/3p1b2/2P5/1P3NP1/P2PPP1P/RNBQKB1R w KQkq - 2 5",
    "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2P2N2/PP1P1PPP/RNBQK2R w KQkq - 3 5",
    "rnbqkbnr/pp3pp1/4p2p/2pp4/4PB2/3P1N2/PPP2PPP/RN1QKB1R w KQkq - 0 5",
    "rn1qkbnr/ppp2ppp/4p3/3p4/8/1P3P2/PBPP1PPP/RN1QKB1R w KQkq - 0 5",
    "r1bqkb1r/pp1npppp/5n2/2pp4/1P6/P4N2/1BPPPPPP/RN1QKB1R w KQkq - 3 5",
    "rnbqkb1r/ppp2ppp/4pn2/8/2pP4/5N2/PP1NPPPP/R1BQKB1R w KQkq - 0 5",
    "rn1qkb1r/ppp2ppp/4pn2/3p1b2/8/3P1NP1/PPPNPP1P/R1BQKB1R w KQkq - 2 5",
    "rnbqk1nr/pp2ppbp/3p2p1/2p5/3P1B2/4PN2/PPP2PPP/RN1QKB1R w KQkq - 0 5",
    "rnbqk1nr/1pp1bppp/4p3/p2p4/3P1B2/5NP1/PPP1PP1P/RN1QKB1R w KQkq - 2 5",
    "rnbqkb1r/pp3ppp/5n2/2Ppp3/2P5/6P1/PP2PP1P/RNBQKBNR w KQkq - 0 5",
    "rnbqkbnr/3p1ppp/p1p1p3/1p6/2P5/PQ3N2/1P1PPPPP/RNB1KB1R w KQkq - 0 5",
    "r1bqkb1r/1ppp1ppp/p1n2n2/4p3/2P1P3/3PB3/PP3PPP/RN1QKBNR w KQkq - 2 5",
    "r1bqkb1r/ppp2ppp/2np1n2/4p3/4P3/P2P1N2/1PP2PPP/RNBQKB1R w KQkq - 0 5",
    "rnbqkb1r/pp1pnp1p/2p3p1/4p3/2P5/3P2P1/PP1NPP1P/R1BQKBNR w KQkq - 0 5",
    "r1bqk2r/pppp1ppp/2n2n2/4p3/1bP5/2NBP3/PP1P1PPP/R1BQK1NR w KQkq - 3 5",
    "rnbqk2r/pppp1ppp/5n2/4p1B1/1bP5/3P1N2/PP2PPPP/RN1QKB1R w KQkq - 5 5",
    "r1bqkb1r/pp1p1ppp/2n2n2/2p1p3/4P3/3P1N2/PPP1BPPP/RNBQK2R w KQkq - 2 5",
    "rnbqk2r/pppp1ppp/2n5/4p3/1bP1P3/3PB3/PP3PPP/RN1QKBNR w KQkq - 3 5",
    "rn1qkbnr/pp3ppp/2p5/3ppb2/2P5/P3P3/1P1PBPPP/RNBQK1NR w KQkq - 0 5",
    "rn1qkbnr/ppp2ppp/8/3p1b2/8/5NP1/PP1PPP1P/RNBQKB1R w KQkq - 0 5",
    "rn1qk1nr/pbppbppp/1p2p3/8/4P3/5N2/PPPPBPPP/RNBQK2R w KQkq - 1 5",
    "rn1qkbnr/1p2pppp/2p5/p2p1b2/8/P2P1NP1/1PP1PP1P/RNBQKB1R w KQkq - 0 5",
    "rnbqkb1r/1p1ppppp/p4n2/2p5/4P3/2PP4/PPQ2PPP/RNB1KBNR w KQkq - 1 5",
    "r1bqkbnr/pp1n1ppp/2p1p3/3p4/2P5/4PNP1/PP1P1P1P/RNBQKB1R w KQkq - 1 5",
    "r1bqkb1r/pppp1pp1/2n2n1p/4p3/2P5/P2P2P1/1P2PP1P/RNBQKBNR w KQkq - 0 5",
    "r1bqkbnr/pp1p1pp1/2n4p/2p1p3/8/2P1P2P/PP1PBPP1/RNBQK1NR w KQkq - 0 5",
    "r1bqk1nr/pp1pbppp/2n1p3/2p5/2P1P3/2N3P1/PP1P1P1P/R1BQKBNR w KQkq - 1 5",
    "rnbqkb1r/p1p2ppp/1p3n2/3pp3/2P5/PP2P3/3P1PPP/RNBQKBNR w KQkq - 0 5",
    "r1bqkbnr/pp3ppp/2n5/2ppp3/4P3/3P1N2/PPP1BPPP/RNBQK2R w KQkq - 0 5",
    "rnbqkb1r/pp2pp1p/2pp1np1/8/3P4/1P2P3/P1PN1PPP/R1BQKBNR w KQkq - 0 5",
    "rnbqkbnr/1p2ppp1/p1p4p/3p4/8/1P1PPN2/P1P2PPP/RNBQKB1R w KQkq - 0 5",
    "rnbqkbnr/2p2ppp/pp2p3/3p4/4PP2/P1N5/1PPP2PP/R1BQKBNR w KQkq - 0 5",
    "r1bqk1nr/pppn1ppp/4p3/3p4/1b6/1P1PPN2/P1P2PPP/RNBQKB1R w KQkq - 1 5",
    "r1bqk1nr/pppn1ppp/3bp3/3p4/8/3PPN1P/PPP2PP1/RNBQKB1R w KQkq - 1 5",
    "rnbqk1nr/1pp2ppp/p2bp3/3p4/3P1B2/P1P5/1P2PPPP/RN1QKBNR w KQkq - 1 5",
    "rnbqk2r/p1ppbppp/1p2pn2/8/2P5/1P2PN2/P2P1PPP/RNBQKB1R w KQkq - 3 5",
    "r1bqk1nr/ppppb1pp/2n5/4pp2/2P5/1Q2P3/PP1PBPPP/RNB1K1NR w KQkq - 0 5",
    "rn1qkbnr/1bpp1ppp/p3p3/1p6/4P3/2PP4/PP2QPPP/RNB1KBNR w KQkq - 1 5",
    "r1bqkbnr/pp3ppp/2np4/2p1p3/2P1P3/P7/1P1PNPPP/RNBQKB1R w KQkq - 1 5",
    "r1bqk1nr/ppp2ppp/2np4/2b1p3/4P3/2NP1N2/PPP2PPP/R1BQKB1R w KQkq - 1 5",
    "rnbqk1nr/p2pppbp/1p4p1/2p5/8/P1N1P3/1PPPBPPP/R1BQK1NR w KQkq - 2 5",
    "rnbqk2r/ppp1bppp/3p1n2/4p3/2P5/P4NP1/1P1PPP1P/RNBQKB1R w KQkq - 3 5",
    "rnbqkb1r/ppp2p1p/3p1np1/4p3/4P3/3P1N2/PPP1BPPP/RNBQK2R w KQkq - 2 5",
    "r2qkbnr/ppp2ppp/2npb3/4p3/4P3/2NP4/PPP2PPP/R1BQKBNR w KQkq - 1 5",
    "rn1qkbnr/pp3ppp/2p1p3/3p1b2/8/2N2NP1/PPPPPPBP/R1BQK2R w KQkq - 0 5",
    "r1bqk1nr/pp1pppbp/2n3p1/1Bp5/4P3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 2 5",
    "rnbqk1nr/ppp2ppp/4p3/6B1/1bpP4/5N2/PP2PPPP/RN1QKB1R w KQkq - 2 5",
    "rnbqkb1r/pp2pppp/2p2n2/8/2pP4/2N3P1/PP2PP1P/R1BQKBNR w KQkq - 1 5",
    "rnb1k1nr/pp1pbppp/1q2p3/2p5/3P1B2/2N4P/PPP1PPP1/R2QKBNR w KQkq - 2 5",
    "r1bqkb1r/ppp2ppp/2np1n2/4p3/4P3/2PP4/PP2BPPP/RNBQK1NR w KQkq - 1 5",
    "rn1qkb1r/pp2pppp/2pp1n2/5b2/8/1P2PN2/P1PPBPPP/RNBQK2R w KQkq - 2 5",
    "rnbqk1nr/ppp2ppp/3bp3/3p4/3P1B2/2P4P/PP2PPP1/RN1QKBNR w KQkq - 1 5",
    "r1bqkb1r/pp1ppp1p/2n2np1/2p5/2P5/BP2P3/P2P1PPP/RN1QKBNR w KQkq - 2 5",
    "rn1qkb1r/ppp1pppp/5n2/3p4/3P4/P4bP1/1PP1PP1P/RNBQKB1R w KQkq - 0 5",
    "rn1qkb1r/ppp2ppp/4pn2/3p1b2/2PP4/P5P1/1P2PP1P/RNBQKBNR w KQkq - 0 5",
    "r1bqk1nr/pppp1ppp/1bn5/4p3/1P2P3/2PP4/P4PPP/RNBQKBNR w KQkq - 1 5",
    "r1bqkb1r/p2ppppp/1pn2n2/2p5/4PP2/3P4/PPP1B1PP/RNBQK1NR w KQkq - 0 5",
    "rnbqkbnr/1p2pppp/p7/2ppP3/8/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 0 5",
    "rnbqk1nr/pp2ppbp/3p2p1/2p5/P2P1B2/5N2/1PP1PPPP/RN1QKB1R w KQkq - 0 5",
    "r1bqkbnr/3ppppp/p1n5/1pp1P3/8/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 1 5",
    "rnbqk1nr/ppp1bpp1/4p2p/3p4/4P3/3P2P1/PPPN1P1P/R1BQKBNR w KQkq - 0 5",
    "rn1qkbnr/ppp1ppp1/8/3pN2p/6b1/BP6/P1PPPPPP/RN1QKB1R w KQkq - 0 5",
    "rn1qkbnr/1pp1pp1p/p5p1/3p1b2/3P4/P5P1/1PPNPP1P/R1BQKBNR w KQkq - 2 5",
    "r1bqkbnr/pp3ppp/2npp3/1Bp5/P3P3/2N5/1PPP1PPP/R1BQK1NR w KQkq - 0 5",
    "rnbqk1nr/pp3ppp/2pp4/2b1p3/2P1P3/P1N5/1P1P1PPP/R1BQKBNR w KQkq - 0 5",
    "rnbqkb1r/ppp2p1p/4pnp1/3p2B1/2PP4/5P2/PP2P1PP/RN1QKBNR w KQkq - 0 5",
    "rnbqkb1r/1p2pppp/p1p2n2/3p4/8/BP3N1P/P1PPPPP1/RN1QKB1R w KQkq - 0 5",
    "r1b1kb1r/ppppqppp/2n1pn2/8/4PB2/2PP4/PP3PPP/RN1QKBNR w KQkq - 3 5",
    "rnbqkb1r/pp1p1ppp/4pn2/8/2Pp4/P4N2/1P2PPPP/RNBQKB1R w KQkq - 0 5",
    "rnbqkb1r/ppp3pp/3p1n2/4pp2/2P5/4P1PN/PP1P1P1P/RNBQKB1R w KQkq - 1 5",
    "rnbqkbnr/1p1p1ppp/p3p3/8/3p4/4P2P/PPP1BPP1/RNBQK1NR w KQkq - 0 5",
    "rnbqkb1r/1pp1pppp/p7/3p4/3Pn3/P5B1/1PP1PPPP/RN1QKBNR w KQkq - 3 5",
    "rnbqkbnr/p2p1p1p/1p2p1p1/2p5/8/1P2PNP1/P1PP1P1P/RNBQKB1R w KQkq - 0 5",
    "r1bqkb1r/1ppp1ppp/p1n2n2/4p3/8/P2PP3/1PP1BPPP/RNBQK1NR w KQkq - 1 5",
    "rnbqkbnr/pp3pp1/4p2p/2pp4/8/4PN1P/PPPPBPP1/RNBQK2R w KQkq - 0 5",
    "rnbqk1nr/p2pppbp/1p4p1/2p5/2P5/3P1NP1/PP2PP1P/RNBQKB1R w KQkq - 0 5",
    "rnbqkb1r/1pp1pp1p/3p1np1/p7/3P4/2P3P1/PP1NPP1P/R1BQKBNR w KQkq - 0 5",
    "rnbqkb1r/1p1ppppp/2p5/p2n4/3P1B2/2P3P1/PP2PP1P/RN1QKBNR w KQkq - 2 5",
    "rnbqk1nr/pp3ppp/2p5/3pp3/1bP1P3/5NP1/PP1P1P1P/RNBQKB1R w KQkq - 0 5",
    "r1bqkbnr/pp1n1ppp/2pp4/4p3/8/2N1PN2/PPPPBPPP/R1BQK2R w KQkq - 0 5",
    "rnbqk2r/p1ppppbp/1p3np1/8/8/1P2PNP1/P1PP1P1P/RNBQKB1R w KQkq - 1 5",
    "r1bqkb1r/pppn1ppp/4pn2/3p4/3P4/5NP1/PPPNPP1P/R1BQKB1R w KQkq - 0 5",
    "r1bqkbnr/pp2pppp/2n5/3p4/2pP4/P3P3/1PP1BPPP/RNBQK1NR w KQkq - 0 5",
    "r1bqkb1r/pppp1p1p/2n2np1/4p3/4P3/P1NP4/1PP2PPP/R1BQKBNR w KQkq - 2 5",
    "r1bqkbnr/pp2pp1p/2np2p1/2p5/2P5/4PNP1/PP1P1P1P/RNBQKB1R w KQkq - 1 5",
    "rn1qkbnr/pp2pppp/2p5/3p4/3P2bN/6P1/PPP1PP1P/RNBQKB1R w KQkq - 2 5",
    "r1bqk1nr/ppp1bppp/2n5/1B1pp2Q/8/2N1P3/PPPP1PPP/R1B1K1NR w KQkq - 2 5",
    "r1bqkb1r/pp1p1ppp/2n1pn2/2p5/2P5/4PN1P/PP1P1PP1/RNBQKB1R w KQkq - 1 5",
    "r1bqkb1r/pppn1ppp/4pn2/3p4/3P1B2/5N2/PPPNPPPP/R2QKB1R w KQkq - 2 5",
    "rnbqkb1r/pppp1pp1/5n1p/8/3P4/8/PPP1BPPP/RNBQK1NR w KQkq - 0 5",
    "rnbqkbnr/1pp2pp1/4p2p/p2p4/8/1P1P1NP1/P1P1PP1P/RNBQKB1R w KQkq - 0 5",
    "r1bqkb1r/pp1p1ppp/2n1pn2/2p5/2P5/2N2N1P/PP1PPPP1/R1BQKB1R w KQkq - 1 5",
    "r1bqkbnr/pp2pppp/3p4/2p5/3nP3/2N5/PPPPBPPP/R1BQK1NR w KQkq - 0 5",
    "rn1qkb1r/ppp1pppp/5n2/5b2/2pP4/P5P1/1P2PP1P/RNBQKBNR w KQkq - 0 5",
    "rnbqkb1r/pp3ppp/2p1pn2/3p4/8/2PP1NP1/PP2PP1P/RNBQKB1R w KQkq - 0 5",
    "r1bqkbnr/ppp3pp/2np4/3Npp2/4P3/3P4/PPP2PPP/R1BQKBNR w KQkq - 0 5",
    "rnbqkb1r/p1p2ppp/1p2pn2/3p4/3P4/1P3NP1/P1P1PP1P/RNBQKB1R w KQkq - 0 5",
    "rnbqkb1r/1pp2ppp/p2p1n2/4p3/2P5/P3P3/1PQP1PPP/RNB1KBNR w KQkq - 0 5",
    "rn1qkb1r/pp2pppp/2p2n2/3p1b2/3P4/1P3N2/PBP1PPPP/RN1QKB1R w KQkq - 3 5",
    "r1bqkb1r/pp1npppp/2pp1n2/8/1PP5/P3P3/3P1PPP/RNBQKBNR w KQkq - 1 5",
    "r1bqkb1r/pppn1ppp/4pn2/3p4/8/P3PN2/1PPPBPPP/RNBQK2R w KQkq - 3 5",
    "rnb1kbnr/pp1ppp1p/6p1/2q5/8/4P3/PPPN1PPP/R1BQKBNR w KQkq - 0 5",
    "r1bqkb1r/pppn1ppp/4pn2/3p4/3P4/5NP1/PPPNPP1P/R1BQKB1R w KQkq - 2 5",
    "r2qkb1r/pppnpppp/5n2/3p4/6b1/P3PN2/1PPPBPPP/RNBQK2R w KQkq - 3 5",
    "rnb1kb1r/ppppqp1p/5np1/4p3/4P3/P1N5/1PPP1PPP/R1BQKBNR w KQkq - 1 5",
    "rn1qkb1r/pp2pppp/2p2n2/3p1b2/3P1B2/2P1P3/PP3PPP/RN1QKBNR w KQkq - 0 5",
    "rnbqk1nr/pp1p1ppp/4pb2/2p5/8/1P2P2P/P1PPBPP1/RNBQK1NR w KQkq - 1 5",
    "r1bqk2r/pppp1ppp/2n2n2/2b1p3/4P3/2NP4/PPP1NPPP/R1BQKB1R w KQkq - 5 5",
    "rn1qkb1r/pp2pppp/2p2n2/3p4/6b1/1P2PN2/P1PPBPPP/RNBQK2R w KQkq - 1 5",
    "rnbqkb1r/pp3ppp/2pp1n2/4p3/8/3PPN2/PPP1BPPP/RNBQK2R w KQkq - 0 5",
    "r1bqk1nr/pppp1p1p/2n3p1/2b1p3/2P1P3/3P4/PP2NPPP/RNBQKB1R w KQkq - 3 5",
    "rnbqkb1r/pp2pppp/2p5/3p2B1/P2Pn3/5N2/1PP1PPPP/RN1QKB1R w KQkq - 1 5",
    "r1bqkb1r/ppp1ppp1/2n2n1p/3p4/8/2PP1NP1/PP2PP1P/RNBQKB1R w KQkq - 0 5",
    "rnbqkb1r/pp3ppp/5n2/2ppp3/2P5/PP6/1B1PPPPP/RN1QKBNR w KQkq - 0 5",
    "rnb1kb1r/ppppqp1p/5np1/4p3/8/P3P1P1/1PPPNP1P/RNBQKB1R w KQkq - 1 5",
    "rnbqkb1r/1pp2ppp/p3pn2/3p4/2PP4/3BP3/PP3PPP/RNBQK1NR w KQkq - 0 5",
    "rnbqkbnr/pp3ppp/8/3pp3/8/P1P5/1P1P1PPP/RNBQKBNR w KQkq - 0 5",
    "rnbqkbnr/pp3ppp/4p3/2pp4/P3P3/7P/1PPPNPP1/RNBQKB1R w KQkq - 0 5",
    "r1bqkbnr/pp1p1ppp/4p3/1Bp5/3nP3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 4 5",
    "rnbqkb1r/pp2ppp1/2p2n1p/3p4/3P4/2NQP3/PPP2PPP/R1B1KBNR w KQkq - 2 5",
    "r1bqkb1r/1ppnpppp/p4n2/3p4/3P4/4PN1P/PPP2PP1/RNBQKB1R w KQkq - 0 5",
    "rnbqk2r/ppp2ppp/3bpn2/3p4/3P4/P1P2N2/1P2PPPP/RNBQKB1R w KQkq - 1 5",
    "rnbqkb1r/p1pp1ppp/1p2p3/3nP3/8/2P5/PP1PQPPP/RNB1KBNR w KQkq - 1 5",
    "rnbqkbnr/pp2pp1p/2p3p1/8/3p4/5N1P/PPPPPPP1/RNBQKB1R w KQkq - 0 5",
    "rn1qkb1r/pbpp1ppp/1p2pn2/8/4P3/2PP4/PPQ2PPP/RNB1KBNR w KQkq - 1 5",
    "r1bqkbnr/pp3ppp/2npp3/2p5/4P3/3P3P/PPP1BPP1/RNBQK1NR w KQkq - 1 5",
    "rnbqkb1r/1p1ppp1p/p4np1/2p5/2P5/P3P2P/1P1P1PP1/RNBQKBNR w KQkq - 1 5",
    "rn1qkbnr/pp3ppp/2p1p3/3p4/6b1/1P2PN2/PBPP1PPP/RN1QKB1R w KQkq - 0 5",
    "rnbqkbnr/pp4pp/2pp4/4pp2/2P5/2NP4/PP2PPPP/1RBQKBNR w Kkq - 0 5",
    "rnbqkbnr/2p2ppp/p3p3/1p1p4/4P3/3P3N/PPP1QPPP/RNB1KB1R w KQkq - 0 5",
    "r1bqkb1r/pppn1ppp/4pn2/3p4/2PP4/3BP3/PP3PPP/RNBQK1NR w KQkq - 3 5",
    "rn1qkbnr/ppp2pp1/4p3/3p3p/6b1/1P3NP1/PBPPPP1P/RN1QKB1R w KQkq - 0 5",
    "r1bqkb1r/pp1npppp/2pp1n2/8/3P4/PP2P3/2P2PPP/RNBQKBNR w KQkq - 1 5",
    "rnbqkb1r/p2p1ppp/1pp1pn2/8/2P5/PP3N2/3PPPPP/RNBQKB1R w KQkq - 0 5",
    "rnbqkb1r/pp1p1p1p/2p2np1/4p3/2P5/P3P3/1P1PNPPP/RNBQKB1R w KQkq - 0 5",
    "r1bqkbnr/pp3ppp/2np4/2p1p3/2P5/P1N4P/1P1PPPP1/R1BQKBNR w KQkq - 1 5",
    "rnbqkb1r/pp3ppp/4pn2/2Pp4/8/P4N2/1PP1PPPP/RNBQKB1R w KQkq - 2 5",
    "r1bqk2r/ppppppbp/2n2np1/8/3P4/4BNP1/PPP1PP1P/RN1QKB1R w KQkq - 4 5",
    "r1bqkbnr/pppn1ppp/8/3p4/8/P4N2/1P1PPPPP/RNBQKB1R w KQkq - 0 5",
    "r1bqkbnr/pp3ppp/n1pp4/4p3/2P5/1P2P3/P2PBPPP/RNBQK1NR w KQkq - 0 5",
    "rnbqkb1r/pp2pp1p/2p2np1/3p4/2P4P/2N1P3/PP1P1PP1/R1BQKBNR w KQkq - 0 5",
    "rnbqkb1r/ppp2ppp/4pn2/8/2p5/4PN2/PP1PBPPP/RNBQK2R w KQkq - 2 5",
    "rnbqkb1r/pp2pppp/5n2/2Pp4/8/2N2P2/PPP1P1PP/R1BQKBNR w KQkq - 1 5",
    "rn1qkb1r/pbpppp1p/1p3np1/6B1/3P4/P4N2/1PP1PPPP/RN1QKB1R w KQkq - 0 5",
    "r1bqkbnr/ppp2p1p/2np2p1/4p3/2P5/P3P3/1PQP1PPP/RNB1KBNR w KQkq - 0 5",
    "rn1qkbnr/pb1p1ppp/1p2p3/2p5/2P1P3/5N1P/PP1P1PP1/RNBQKB1R w KQkq - 1 5",
    "rn1qkbnr/pb1ppp1p/1p4p1/2p5/8/2PP1NP1/PP2PP1P/RNBQKB1R w KQkq - 1 5",
    "rnbqkb1r/pp1p1ppp/4pn2/8/2Pp4/P3P3/1P3PPP/RNBQKBNR w KQkq - 0 5",
    "rnbqk2r/ppp1bppp/4pn2/3p4/3P4/P3P2P/1PP2PP1/RNBQKBNR w KQkq - 1 5",
    "rnbqk2r/ppp1bppp/4pn2/3p4/3P4/P4N2/1PPNPPPP/R1BQKB1R w KQkq - 4 5",
    "rn2kbnr/pp2pppp/1q6/2pp1b2/8/1P3N2/PBPPPPPP/RN1QKB1R w KQkq - 4 5",
    "rnbq1rk1/ppppbppp/4pn2/8/3P4/P3P3/1PP1BPPP/RNBQK1NR w KQ - 3 5",
    "r1bqk1nr/ppp2ppp/2np4/4p3/1bP5/1QN2N2/PP1PPPPP/R1B1KB1R w KQkq - 0 5",
    "rnbqkbnr/ppp2ppp/8/8/3p4/2N2N2/PP1PPPPP/R1BQKB1R w KQkq - 0 5",
    "rnbqkb1r/1p2pppp/p1p2n2/3p4/8/BP3N1P/P1PPPPP1/RN1QKB1R w KQkq - 0 5",
    "rnbqk2r/pppp1pp1/5n1p/4p3/1bP5/2NP2P1/PP2PP1P/R1BQKBNR w KQkq - 0 5",
    "rnbqkbnr/pp3ppp/4p3/2p5/1PPp4/P4N2/3PPPPP/RNBQKB1R w KQkq - 0 5",
    "rnb1kbnr/1pq1pppp/2pp4/p7/P7/2N2NP1/1PPPPP1P/R1BQKB1R w KQkq - 0 5",
    "rnbqkb1r/3ppppp/p4n2/1pp5/8/3PP2P/PPP1BPP1/RNBQK1NR w KQkq - 1 5",
    "rnbqk1nr/pp2bppp/2p1p3/3p4/8/P3PN1P/1PPP1PP1/RNBQKB1R w KQkq - 1 5",
    "rn1qk1nr/ppp1bppp/3p4/4pb2/1PP5/P3P3/3P1PPP/RNBQKBNR w KQkq - 1 5",
    "r1bqkbnr/ppp2ppp/2n5/3pp3/4P3/2PP1N2/PP3PPP/RNBQKB1R w KQkq - 0 5",
    "rnbqkb1r/1p1p1ppp/p3pn2/2p5/2P5/P3P2P/1P1P1PP1/RNBQKBNR w KQkq - 0 5",
    "rnbqk1nr/1ppp1ppp/8/p1b1p3/2P5/3P2P1/PP1BPP1P/RN1QKBNR w KQkq - 1 5",
    "r1bqkbnr/ppp2pp1/2n1p2p/3p4/4PB2/3P1N2/PPP2PPP/RN1QKB1R w KQkq - 2 5",
    "rnbqk2r/p1ppbppp/1p2pn2/8/3P1B1P/P7/1PP1PPP1/RN1QKBNR w KQkq - 1 5",
    "r1bqk1nr/ppp1bppp/2np4/4p3/2P1P3/P2B4/1P1P1PPP/RNBQK1NR w KQkq - 2 5",
    "rnbqk1nr/pp2ppbp/2p3p1/3p4/2P1P3/5N1P/PP1P1PP1/RNBQKB1R w KQkq - 0 5",
    "rn1qkb1r/pbp1pppp/1p3n2/3p4/8/4PNP1/PPPPQP1P/RNB1KB1R w KQkq - 0 5",
    "rnbqkbnr/pp2pp1p/2p3p1/8/3p4/5NP1/PPPPPP1P/RNBQKB1R w KQkq - 0 5",
    "rnbqk1nr/pp3ppp/3bp3/2pp4/8/1P2PN2/P1PPBPPP/RNBQK2R w KQkq - 0 5",
    "rn1qkbnr/1pp1ppp1/7p/p2p1b2/3P4/1P3NP1/P1P1PP1P/RNBQKB1R w KQkq - 0 5",
    "r1bqkb1r/pppp1p1p/2n2np1/4p3/1P6/P3P2P/2PP1PP1/RNBQKBNR w KQkq - 0 5",
    "rnbqkb1r/pp3ppp/2pp1n2/4p3/8/P1N1PN2/1PPP1PPP/R1BQKB1R w KQkq - 2 5",
    "rnbqk1nr/p1p2ppp/1p1bp3/3p4/3P4/PP6/1BP1PPPP/RN1QKBNR w KQkq - 2 5",
    "r1b1kbnr/pp2pppp/2n5/qBpp4/3P1B2/4P3/PPP2PPP/RN1QK1NR w KQkq - 3 5",
    "rn1qkb1r/pppbpppp/5n2/3p4/3P3N/P7/1PP1PPPP/RNBQKB1R w KQkq - 5 5",
    "rnbqkb1r/pp3ppp/4pn2/2pp4/3P4/P3P3/1PP1BPPP/RNBQK1NR w KQkq - 0 5",
    "rnbqkb1r/pp2ppp1/2p2n1p/3p4/2PP4/P7/1P1NPPPP/R1BQKBNR w KQkq - 0 5",
    "rnb1k1nr/ppqpbppp/4p3/2p5/4P3/2NP2P1/PPP2P1P/R1BQKBNR w KQkq - 1 5",
    "r1bqkb1r/pp1p1ppp/2n1pn2/2p5/4P3/2P2N2/PPQP1PPP/RNB1KB1R w KQkq - 0 5",
    "rnbqkb1r/pp2pp1p/2p2np1/3p4/1PP5/P3P3/3P1PPP/RNBQKBNR w KQkq - 0 5",
    "rnbqkb1r/1p1p1ppp/2p1pn2/p7/3P4/4P3/PPPBBPPP/RN1QK1NR w KQkq - 0 5",
    "r1bqkbnr/pp1p1ppp/2n5/2p1p3/2P1P3/P4N2/1P1P1PPP/RNBQKB1R w KQkq - 0 5",
    "r1bqkb1r/pppp1ppp/2n2n2/8/2BpP3/5N2/PPP2PPP/RNBQK2R w KQkq - 4 5",
    "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2P5/2NP4/PP1BPPPP/R2QKBNR w KQkq - 5 5",
    "r1bqkb1r/ppp1pppp/2n2n2/3p4/3P4/5PB1/PPP1P1PP/RN1QKBNR w KQkq - 0 5",
    "rnbqkb1r/pp3ppp/4pn2/2Pp4/8/4P3/PPP1BPPP/RNBQK1NR w KQkq - 0 5",
    "rn1qkbnr/pp2pppp/2p3b1/3p4/3P4/P1N4P/1PP1PPP1/R1BQKBNR w KQkq - 1 5",
    "r1bqk2r/ppppbppp/2n1pn2/1B6/4P3/3P4/PPP1NPPP/RNBQK2R w KQkq - 2 5",
    "r1bqkb1r/pp1p1ppp/2n1pn2/2p5/3P1B2/2P5/PP1NPPPP/R2QKBNR w KQkq - 2 5",
    "r1bqkb1r/pp2pppp/2n4n/2pp4/4PP2/3P4/PPP1B1PP/RNBQK1NR w KQkq - 0 5",
    "rn1qkb1r/pp2pppp/3p1n2/2p2b2/2P5/BP3N2/P2PPPPP/RN1QKB1R w KQkq - 1 5",
    "rnbq1rk1/ppppbppp/4pn2/8/8/P3PN2/1PPPBPPP/RNBQK2R w KQ - 1 5",
    "r2qkbnr/pppn1ppp/4p3/3p1b2/3P4/P2BP3/1PP2PPP/RNBQK1NR w KQkq - 0 5",
    "rnbqk1nr/p1p1bppp/1p2p3/3p4/2P5/PP3N2/3PPPPP/RNBQKB1R w KQkq - 1 5",
    "rnbqk2r/ppp2ppp/3bpn2/3p4/8/2P2NP1/PP1PPPBP/RNBQK2R w KQkq - 2 5",
    "r1bqk1nr/pp1pbppp/2n5/2p1p3/2P5/2N2N1P/PP1PPPP1/R1BQKB1R w KQkq - 2 5",
    "rnbqkb1r/1pp2ppp/4pn2/p2p4/8/1P2PNP1/P1PP1P1P/RNBQKB1R w KQkq - 0 5",
    "r1bqkb1r/pp1npppp/5n2/2pp2B1/3P4/2P2N2/PP2PPPP/RN1QKB1R w KQkq - 1 5",
    "rnbqkb1r/pp3ppp/2p1pn2/3p4/8/P1N2NP1/1PPPPP1P/R1BQKB1R w KQkq - 0 5",
    "rnbqkbnr/pp2pppp/8/2p5/3p4/P4N2/1PPPPPPP/RNBQKB1R w KQkq - 0 5",
    "rn1qkbnr/1bpp1ppp/p3p3/1p6/3P4/PP2P3/2P2PPP/RNBQKBNR w KQkq - 1 5",
    "r1bqkb1r/pp1pnppp/2n1p3/2p5/2P1P3/2N3P1/PP1P1P1P/R1BQKBNR w KQkq - 1 5",
    "rnbqk2r/ppp1bppp/4pn2/3p4/3P1B2/P4N2/1PP1PPPP/RN1QKB1R w KQkq - 4 5",
    "rnbqkbnr/1p1p1ppp/p3p3/8/2Pp4/1P2P3/P4PPP/RNBQKBNR w KQkq - 0 5",
    "rnb1kb1r/pp1pqppp/2p2n2/4p3/8/PP2P3/1BPP1PPP/RN1QKBNR w KQkq - 2 5",
    "rn1qkb1r/pbpppppp/1p6/8/3Pn3/4PB2/PPP2PPP/RNBQK1NR w KQkq - 3 5",
    "rnbqk1nr/ppp1bppp/8/3pp3/8/P1N1P3/1PPPNPPP/R1BQKB1R w KQkq - 0 5",
    "r1bqkb1r/pppn1ppp/4pn2/3p4/3P4/1P3N2/P1PNPPPP/R1BQKB1R w KQkq - 2 5",
    "rnb1kb1r/ppp1qppp/3p1n2/4p3/2P5/P3PN2/1P1P1PPP/RNBQKB1R w KQkq - 0 5",
    "rnbqkbnr/1pp2ppp/p7/3p4/8/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 0 5",
    "rnbqkb1r/1p1p1ppp/p3pn2/2p5/2P5/5NPP/PP1PPP2/RNBQKB1R w KQkq - 0 5",
    "rnbqkb1r/p1p2ppp/1p2pn2/3p4/2P5/P2P1N2/1P2PPPP/RNBQKB1R w KQkq - 0 5",
    "rnbqkb1r/1p1p1ppp/p4n2/2p1p3/2P1P3/P2P4/1P3PPP/RNBQKBNR w KQkq - 0 5",
    "r1bqkb1r/1ppp1ppp/2n2n2/p3p3/2P5/1QN1P3/PP1P1PPP/R1B1KBNR w KQkq - 0 5",
    "rnbqk1nr/p1p1bppp/1p2p3/3p4/2P5/P3PN2/1P1P1PPP/RNBQKB1R w KQkq - 1 5",
    "rn1qkb1r/ppp2ppp/4pn2/3p1b2/3P4/3BPN2/PPP2PPP/RNBQK2R w KQkq - 0 5",
    "rnbqk2r/pp1pbppp/4pn2/2p5/2P5/1P4P1/P2PPPBP/RNBQK1NR w KQkq - 2 5",
    "rnbqk2r/ppp1bppp/4pn2/3p4/3P1B2/P1N5/1PP1PPPP/R2QKBNR w KQkq - 1 5",
    "rnbqk1nr/pp1pb1pp/2p5/4pp2/2P5/P3P3/1P1PNPPP/RNBQKB1R w KQkq - 0 5",
    "r1bqkbnr/ppp2ppp/2n1p3/8/4p3/P1NP4/1PP2PPP/R1BQKBNR w KQkq - 1 5",
    "rnq1kbnr/ppp2ppp/3p4/4pb2/3P4/P3PQ2/1PP2PPP/RNB1KBNR w KQkq - 3 5",
    "r1bqkbnr/1p1p1ppp/p1n1p3/2p5/4P3/N1P2N2/PP1P1PPP/R1BQKB1R w KQkq - 2 5",
    "r1bqkb1r/pppp1ppp/5n2/4p3/2Pn4/2N2NP1/PP1PPP1P/R1BQKB1R w KQkq - 1 5",
    "rn1qkb1r/ppp2ppp/4pn2/3p1b2/2P5/PQ2P3/1P1P1PPP/RNB1KBNR w KQkq - 2 5",
    "r1bqkb1r/ppp1ppp1/2n2n2/3p3p/3P4/2N2NP1/PPP1PP1P/R1BQKB1R w KQkq - 0 5",
    "rnbqkb1r/ppp2ppp/4pn2/8/2p5/4PN2/PPQP1PPP/RNB1KB1R w KQkq - 2 5",
    "rnbqk1nr/1pp1bppp/3p4/p3p3/2P2P2/P2P4/1P2P1PP/RNBQKBNR w KQkq - 0 5",
    "rnbqkbnr/pp3p1p/3p2p1/2p1p3/2P5/2N2N1P/PP1PPPP1/R1BQKB1R w KQkq - 0 5",
    "rnb1kb1r/pp2pppp/2p2n2/q2p4/3P4/1P3NP1/P1P1PP1P/RNBQKB1R w KQkq - 1 5",
    "r1bqk1nr/pppp1ppp/1bn5/4p3/4P3/P2P4/1PP1BPPP/RNBQK1NR w KQkq - 3 5",
    "rnbqkb1r/pp3ppp/3ppn2/2p5/4P3/2NP2P1/PPP2P1P/R1BQKBNR w KQkq - 1 5",
    "r1bqkb1r/pp1pnppp/2n5/2p1p3/2P5/2NP3P/PP2PPP1/R1BQKBNR w KQkq - 1 5",
    "rnbqkb1r/pp2ppp1/2pp1n1p/8/1P6/P3P3/2PPBPPP/RNBQK1NR w KQkq - 0 5",
    "r1bqkb1r/pp1pppp1/2n2n1p/2p5/2P5/4PNP1/PP1P1P1P/RNBQKB1R w KQkq - 2 5",
    "rnb1kb1r/pp1p1ppp/2p2n2/q3p3/2P5/4PNP1/PP1P1P1P/RNBQKB1R w KQkq - 1 5",
    "rnbqkbnr/p2p1ppp/4p3/1p6/3Q4/5NP1/PPP1PP1P/RNB1KB1R w KQkq - 0 5",
    "rnbqk1nr/1pp2ppp/3p4/p1b1p3/4P3/3P1NP1/PPP2P1P/RNBQKB1R w KQkq - 0 5",
    "rnbqk2r/1pppbppp/4pn2/p7/8/P1N2NP1/1PPPPP1P/R1BQKB1R w KQkq - 1 5",
    "r1bqkbnr/pp3ppp/2n1p3/2pp4/8/P3PN2/1PPPBPPP/RNBQK2R w KQkq - 0 5",
    "rnbqkb1r/1p2pppp/p2p1n2/2p5/3P4/3BP3/PPPN1PPP/R1BQK1NR w KQkq - 0 5",
    "rn1qkbnr/pbp2ppp/1p2p3/3p4/3P4/1P1BP3/P1P2PPP/RNBQK1NR w KQkq - 1 5",
    "rnbqkb1r/pppp1p1p/5np1/8/3p4/P3P3/1PP1BPPP/RNBQK1NR w KQkq - 0 5",
    "r1bqkb1r/pp1ppp1p/2n2np1/2p5/2P5/1QN1P3/PP1P1PPP/R1B1KBNR w KQkq - 4 5",
    "rnbqk2r/ppppbppp/4pn2/8/2PP4/8/PP1BPPPP/RN1QKBNR w KQkq - 5 5",
    "rnbqk1nr/ppp1bppp/8/3pp3/4P3/3P4/PPP1BPPP/RNBQK1NR w KQkq - 0 5",
    "rn1qkbnr/pb1p1ppp/1p2p3/2p5/2P5/P2P1N2/1P2PPPP/RNBQKB1R w KQkq - 0 5",
    "rnbqkbnr/pp3ppp/3p4/2p1p3/4P3/2PP4/PP2QPPP/RNB1KBNR w KQkq - 0 5",
    "rnbqk1nr/ppp2ppp/3b4/3p4/3p4/2N1P2P/PPP2PP1/R1BQKBNR w KQkq - 0 5",
    "r1bqkbnr/pp1n1ppp/2pp4/4p3/8/2N1PN1P/PPPP1PP1/R1BQKB1R w KQkq - 0 5",
    "r1bqk1nr/pppn1ppp/4p3/3p4/1bPP4/4P2P/PP3PP1/RNBQKBNR w KQkq - 1 5",
    "rnbqkbnr/pp3ppp/2p1p3/8/3pP3/2NP4/PPP1QPPP/R1B1KBNR w KQkq - 0 5",
    "rnbqk1nr/1pp1bppp/p2pp3/8/4P3/3P4/PPP1BPPP/RNBQK1NR w KQkq - 0 5",
    "r1bqkbnr/pp3ppp/2np4/2p1p3/4P3/3P2P1/PPP2PBP/RNBQK1NR w KQkq - 2 5",
    "rn1qkbnr/pp1b1ppp/4p3/2pp4/3P1B2/P1P5/1P2PPPP/RN1QKBNR w KQkq - 0 5",
    "r1bqkbnr/1ppn1ppp/p2p4/4p3/4P3/P1N5/1PPP1PPP/R1BQKBNR w KQkq - 0 5",
    "rn1qkb1r/pp2pppp/2pp1n2/5b2/2P5/1P3N1P/P2PPPP1/RNBQKB1R w KQkq - 1 5",
    "rnbqk2r/ppp1bppp/3ppn2/8/3P4/4P3/PPPNBPPP/R1BQK1NR w KQkq - 0 5",
    "r1bqkbnr/ppp3pp/2np4/4pp2/4P3/P1NP4/1PP2PPP/R1BQKBNR w KQkq - 0 5",
    "r1bqkb1r/pppn1ppp/3ppn2/8/3P1B2/2P3P1/PP2PP1P/RN1QKBNR w KQkq - 2 5",
    "r1bqkbnr/p2p1ppp/1pn1p3/2p5/4P3/2N4P/PPPPBPP1/R1BQK1NR w KQkq - 2 5",
    "r1bqk1nr/pppnppbp/6p1/3p4/8/P2P1NP1/1PP1PP1P/RNBQKB1R w KQkq - 1 5",
    "rnbqkb1r/ppp3pp/3p1n2/4pp2/2P5/P1NP4/1P2PPPP/R1BQKBNR w KQkq - 2 5",
    "rnbqk2r/ppppbpp1/4pn1p/8/8/3P1NP1/PPP1PPBP/RNBQK2R w KQkq - 0 5",
    "rnbqk2r/ppp1ppbp/3p1np1/8/3P4/2P3P1/PP2PPBP/RNBQK1NR w KQkq - 2 5",
    "rnbqkbnr/1p2ppp1/p1p4p/3p4/8/P1N2NP1/1PPPPP1P/R1BQKB1R w KQkq - 0 5",
];
