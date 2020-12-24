use criterion::{BatchSize, Criterion, black_box};

use toon::Grid;

fn main() {
    let mut c = Criterion::default().configure_from_args();

    c.bench_function("Grid::clear", |b| {
        b.iter_batched(
            || Grid::new((320, 96)),
            |mut grid| {
                grid.clear();
                black_box(grid)
            },
            BatchSize::LargeInput,
        );
    });
}
