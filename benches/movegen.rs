use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use redtail::board::{Board, Position};
use redtail::movegen::MoveGenerator;

// Test positions for benchmarking
const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
const POSITION_4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
const POSITION_6: &str = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";

fn bench_generate_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_moves");

    let positions = [
        ("starting", STARTING_FEN),
        ("kiwipete", KIWIPETE),
        ("position_3", POSITION_3),
        ("position_4", POSITION_4),
        ("position_5", POSITION_5),
        ("position_6", POSITION_6),
    ];

    for (name, fen) in positions.iter() {
        group.bench_with_input(BenchmarkId::new("all_moves", name), fen, |b, fen| {
            let mut board = <Position as Board>::new(Some(fen));
            b.iter(|| black_box(board.generate_moves(false)))
        });
    }

    for (name, fen) in positions.iter() {
        group.bench_with_input(BenchmarkId::new("captures_only", name), fen, |b, fen| {
            let mut board = <Position as Board>::new(Some(fen));
            b.iter(|| black_box(board.generate_moves(true)))
        });
    }

    group.finish();
}

fn bench_generate_legal_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_legal_moves");

    let positions = [
        ("starting", STARTING_FEN),
        ("kiwipete", KIWIPETE),
        ("position_3", POSITION_3),
        ("position_4", POSITION_4),
        ("position_5", POSITION_5),
        ("position_6", POSITION_6),
    ];

    for (name, fen) in positions.iter() {
        group.bench_with_input(BenchmarkId::new("legal", name), fen, |b, fen| {
            let mut board = <Position as Board>::new(Some(fen));
            b.iter(|| black_box(board.generate_legal_moves()))
        });
    }

    group.finish();
}

fn bench_individual_piece_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("piece_moves");

    // Use kiwipete position - it has all piece types active
    let mut board = <Position as Board>::new(Some(KIWIPETE));
    let mut moves = Vec::with_capacity(256);

    group.bench_function("pawns", |b| {
        b.iter(|| {
            moves.clear();
            board.generate_pawn_moves(&mut moves, false);
            black_box(&moves);
        })
    });

    group.bench_function("knights", |b| {
        b.iter(|| {
            moves.clear();
            board.generate_knight_moves(&mut moves, false);
            black_box(&moves);
        })
    });

    group.bench_function("bishops", |b| {
        b.iter(|| {
            moves.clear();
            board.generate_bishop_moves(&mut moves, false);
            black_box(&moves);
        })
    });

    group.bench_function("rooks", |b| {
        b.iter(|| {
            moves.clear();
            board.generate_rook_moves(&mut moves, false);
            black_box(&moves);
        })
    });

    group.bench_function("queens", |b| {
        b.iter(|| {
            moves.clear();
            board.generate_queen_moves(&mut moves, false);
            black_box(&moves);
        })
    });

    group.bench_function("king", |b| {
        b.iter(|| {
            moves.clear();
            board.generate_king_moves(&mut moves, false);
            black_box(&moves);
        })
    });

    group.bench_function("castling", |b| {
        b.iter(|| {
            moves.clear();
            board.generate_castle_moves(&mut moves);
            black_box(&moves);
        })
    });

    group.finish();
}

fn bench_perft(c: &mut Criterion) {
    let mut group = c.benchmark_group("perft");
    group.sample_size(10); // Perft is slow, reduce sample size

    let mut board = <Position as Board>::new(Some(STARTING_FEN));

    group.bench_function("depth_1", |b| {
        b.iter(|| black_box(board.perft(1)))
    });

    group.bench_function("depth_2", |b| {
        b.iter(|| black_box(board.perft(2)))
    });

    group.bench_function("depth_3", |b| {
        b.iter(|| black_box(board.perft(3)))
    });

    group.bench_function("depth_4", |b| {
        b.iter(|| black_box(board.perft(4)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_generate_moves,
    bench_generate_legal_moves,
    bench_individual_piece_moves,
    bench_perft,
);

criterion_main!(benches);
