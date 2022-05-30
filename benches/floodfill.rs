use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jca;

pub fn empty_bench(c: &mut Criterion) {
    let empty_board = jca::Board::new();

    let mut group_empty = c.benchmark_group("Empty Board");

    group_empty.bench_function("top left",     |b| b.iter(|| empty_board.click(black_box(&(0, 0)))));
    group_empty.bench_function("top right",    |b| b.iter(|| empty_board.click(black_box(&(7, 0)))));
    group_empty.bench_function("bottom left",  |b| b.iter(|| empty_board.click(black_box(&(0, 7)))));
    group_empty.bench_function("bottom right", |b| b.iter(|| empty_board.click(black_box(&(7, 7)))));
    group_empty.bench_function("middle",       |b| b.iter(|| empty_board.click(black_box(&(3, 3)))));
    group_empty.finish();
}

pub fn filled_bench(c: &mut Criterion) {

    let filled_board = jca::Board::from_rows([
        0x_3_2_3_3_2_2_4_3,
        0x_2_4_3_3_3_3_3_8,
        0x_2_2_3_3_3_2_2_3,
        0x_2_3_2_2_3_4_2_2,
        0x_2_4_2_3_3_3_3_2,
        0x_2_2_2_2_2_2_4_2,
        0x_3_4_3_3_2_3_2_3,
        0x_2_4_2_2_2_4_3_3,
    ]);

    let mut group_filled = c.benchmark_group("Filled Board");
    group_filled.bench_function("top left", |b| b.iter(|| filled_board.click(black_box(&(0, 0)))));
    group_filled.bench_function("top right", |b| b.iter(|| filled_board.click(black_box(&(7, 0)))));
    group_filled.bench_function("bottom left", |b| b.iter(|| filled_board.click(black_box(&(0, 7)))));
    group_filled.bench_function("bottom right", |b| b.iter(|| filled_board.click(black_box(&(7, 7)))));
    group_filled.bench_function("middle", |b| b.iter(|| filled_board.click(black_box(&(3, 3)))));
    group_filled.finish();
}


pub fn click_bench(c: &mut Criterion) {
    let board = jca::Board::from_rows([
        0x_3_2_3_3_2_2_4_3,
        0x_2_4_3_3_3_3_3_8,
        0x_2_2_3_3_3_2_2_3,
        0x_2_3_2_2_3_4_2_2,
        0x_2_4_2_3_3_3_3_2,
        0x_2_2_2_2_2_2_4_2,
        0x_3_4_3_3_2_3_2_3,
        0x_2_4_2_2_2_4_3_3,
    ]);

    let mut group_click = c.benchmark_group("Click");
    for (x, y) in [(0, 0), (0, 1), (0, 2), (0, 3), (2, 3), (2, 4), (5, 6), (2, 7), (6, 4)] { //0..jca::BOARD_SIZE {
        // for y in 0..jca::BOARD_SIZE {
            group_click.bench_function(format!("floodfill {x} {y}"), |b| b.iter(|| board.click(black_box(&(x, y)))));
            group_click.bench_function(format!("simdflood {x} {y}"), |b| b.iter(|| unsafe { board.click3(black_box((x, y))) }));
        // }
    }

    group_click.finish();
}

pub fn collapse_bench(c: &mut Criterion) {
    let mut board1 = jca::Board::from_rows([
        0x_3_2_0_0_2_2_0_3,
        0x_2_4_0_0_3_3_0_8,
        0x_0_2_0_0_3_2_0_0,
        0x_0_3_0_0_3_0_2_0,
        0x_2_4_0_3_3_0_3_0,
        0x_0_2_0_2_2_0_4_0,
        0x_3_4_0_0_2_0_2_0,
        0x_0_4_0_0_2_0_3_0,
    ]);

    let mut board2 = board1.clone();

    let mut group = c.benchmark_group("Collapse");
    // group.bench_function(format!("collapse"),       |b| b.iter(|| board1.collapse()));
    // group.bench_function(format!("collapse_simd"),  |b| b.iter(|| unsafe { board2.collapse_simd() }));
    group.bench_function(format!("collapse_simd2"), |b| b.iter(|| unsafe { board2.collapse_simd2() }));

    group.finish();
}

// criterion_group!(benches, click_bench); //empty_bench, filled_bench);
criterion_group!(benches, collapse_bench);
criterion_main!(benches);