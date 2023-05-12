use uci::UCI;

mod board;
mod chess;
mod evaluation;
mod movegen;
mod uci;
mod utils;
mod tt;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let mut uci = UCI::new();
    uci.uci_loop();
}
