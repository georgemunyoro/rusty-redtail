use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use redtail::board::{Board, Position};
use redtail::search::evaluate::Evaluator;

// Test positions for benchmarking - variety of game phases and complexity
const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
const MIDDLEGAME: &str = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
const ENDGAME: &str = "8/8/4k3/8/8/4K3/4P3/8 w - - 0 1";
const COMPLEX_PAWNS: &str = "8/pp3ppp/2p5/3p4/3P4/2P5/PP3PPP/8 w - - 0 1";
const ROOK_ENDGAME: &str = "8/5k2/8/8/8/8/R4K2/8 w - - 0 1";
const OPEN_FILES: &str = "r3k2r/ppp2ppp/8/8/8/8/PPP2PPP/R3K2R w KQkq - 0 1";

fn bench_evaluate_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluate");

    let positions = [
        ("starting", STARTING_FEN),
        ("kiwipete", KIWIPETE),
        ("middlegame", MIDDLEGAME),
        ("endgame", ENDGAME),
        ("complex_pawns", COMPLEX_PAWNS),
        ("rook_endgame", ROOK_ENDGAME),
        ("open_files", OPEN_FILES),
    ];

    for (name, fen) in positions.iter() {
        group.bench_with_input(BenchmarkId::new("full", name), fen, |b, fen| {
            let mut position = Position::new(Some(fen));
            let mut evaluator = Evaluator::new();
            b.iter(|| black_box(evaluator.evaluate(&mut position)))
        });
    }

    group.finish();
}

fn bench_evaluate_pawn_structure(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluate_pawn_structure");

    let positions = [
        ("starting", STARTING_FEN),
        ("kiwipete", KIWIPETE),
        ("complex_pawns", COMPLEX_PAWNS),
        ("endgame", ENDGAME),
    ];

    for (name, fen) in positions.iter() {
        group.bench_with_input(BenchmarkId::new("pawns", name), fen, |b, fen| {
            let mut position = Position::new(Some(fen));
            let mut evaluator = Evaluator::new();
            b.iter(|| black_box(evaluator.evaluate_pawn_structure(&mut position)))
        });
    }

    group.finish();
}

fn bench_evaluate_open_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluate_open_files");

    let positions = [
        ("starting", STARTING_FEN),
        ("kiwipete", KIWIPETE),
        ("open_files", OPEN_FILES),
        ("rook_endgame", ROOK_ENDGAME),
    ];

    for (name, fen) in positions.iter() {
        group.bench_with_input(BenchmarkId::new("rooks", name), fen, |b, fen| {
            let mut position = Position::new(Some(fen));
            let mut evaluator = Evaluator::new();
            b.iter(|| black_box(evaluator.evaluate_open_files(&mut position)))
        });
    }

    group.finish();
}

fn bench_evaluate_king_safety(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluate_king_safety");

    let positions = [
        ("starting", STARTING_FEN),
        ("kiwipete", KIWIPETE),
        ("middlegame", MIDDLEGAME),
        ("endgame", ENDGAME),
    ];

    for (name, fen) in positions.iter() {
        group.bench_with_input(BenchmarkId::new("king", name), fen, |b, fen| {
            let mut position = Position::new(Some(fen));
            let mut evaluator = Evaluator::new();
            b.iter(|| black_box(evaluator.evaluate_king_safety(&mut position)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_evaluate_full,
    bench_evaluate_pawn_structure,
    bench_evaluate_open_files,
    bench_evaluate_king_safety,
);

criterion_main!(benches);
