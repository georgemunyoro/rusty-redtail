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

fn main() {
    let mut uci = UCI::new();
    uci.uci_loop();
}
