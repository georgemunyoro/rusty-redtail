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
    let mut uci = UCI::new();
    uci.uci_loop();

    // let mut evaluator = evaluation::Evaluator::new();
    // let mut position = board::Position::new(None);
    // position.set_fen(String::from("7k/8/8/P1P5/5p1p/8/8/K7 w - - 0 1"));
    // position.draw();
    // println!("{}", evaluator.evaluate(&mut position));
}
