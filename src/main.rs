mod uci;
use uci::UCI;

fn main() {
    let mut u = UCI::new();
    u.uci_loop();
}
