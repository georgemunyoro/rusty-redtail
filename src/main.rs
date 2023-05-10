mod board;
mod chess;
mod evaluation;
mod movegen;
mod uci;
mod utils;

fn main() {
    let mut uci = uci::UCI::new();
    uci.uci_loop();
}
