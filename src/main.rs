use uci::UCI;

mod board;
mod chess;
mod evaluation;
mod movegen;
mod uci;
mod utils;

fn main() {
    let mut uci = UCI::new();
    uci.uci_loop();
}
