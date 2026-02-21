mod uci;
use uci::UCI;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut u = UCI::new();

    if args.len() > 1 && args[1] == "bench" {
        let tokens: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();
        u.bench(tokens);
        return;
    }

    u.uci_loop();
}
