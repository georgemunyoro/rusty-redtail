use board::Board;
use uci::UCI;

mod board;
mod chess;
mod evaluation;
mod movegen;
mod pst;
mod skaak;
mod tt;
mod uci;
mod utils;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    // let mut uci = UCI::new();
    // uci.uci_loop();

    let mut evaluator = evaluation::Evaluator::new();
    let mut position = board::Position::new(None);
    position.set_fen(String::from("7k/p6p/p6p/8/8/P7/PPP5/7K w - - 0 1"));
    position.draw();
    println!("{}", evaluator.evaluate(&mut position));
}
