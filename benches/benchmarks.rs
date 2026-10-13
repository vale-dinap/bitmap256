use bitmap256::Bitmap256;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_from_bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");

    for bit in [0, 63, 127, 255] {
        group.bench_with_input(BenchmarkId::new("from_bit", bit), &bit, |b, &i| {
            b.iter(|| Bitmap256::from_bit(black_box(i)))
        });
    }

    group.bench_function("from_value", |b| {
        b.iter(|| Bitmap256::from_value(black_box(0xFFu128)))
    });

    group.bench_function("new", |b| {
        b.iter(Bitmap256::new)
    });

    group.finish();
}

fn bench_set_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("mutation");

    for bit in [0, 63, 127, 255] {
        group.bench_with_input(BenchmarkId::new("set_bit", bit), &bit, |b, &i| {
            let mut bm = Bitmap256::new();
            b.iter(|| {
                bm.set_bit(black_box(i));
                bm
            })
        });

        group.bench_with_input(BenchmarkId::new("clear_bit", bit), &bit, |b, &i| {
            let mut bm = Bitmap256::from_bit(i);
            b.iter(|| {
                bm.clear_bit(black_box(i));
                bm
            })
        });
    }

    group.finish();
}

fn bench_flip(c: &mut Criterion) {
    let mut group = c.benchmark_group("mutation");

    for bit in [0, 63, 127, 255] {
        group.bench_with_input(BenchmarkId::new("flip_bit", bit), &bit, |b, &i| {
            let mut bm = Bitmap256::from_bit(i);
            b.iter(|| {
                bm.flip_bit(black_box(i));
                bm
            })
        });
    }

    group.finish();
}

fn bench_count_ones(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis");

    let sparse = Bitmap256::from_value(0b101u128);
    let dense = Bitmap256::from_value(u64::MAX as u128);
    let full = !Bitmap256::new();

    for (name, bm) in [("sparse", sparse), ("dense", dense), ("full", full)] {
        group.bench_with_input(BenchmarkId::new("count_ones", name), &bm, |b, m| {
            b.iter(|| black_box(m).count_ones())
        });
    }

    group.finish();
}

fn bench_zeros(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis");

    let sparse = Bitmap256::from_value(0b101u128);
    let dense = Bitmap256::from_value(u64::MAX as u128);
    let full = !Bitmap256::new();

    for (name, bm) in [("sparse", sparse), ("dense", dense), ("full", full)] {
        group.bench_with_input(BenchmarkId::new("leading_zeros", name), &bm, |b, m| {
            b.iter(|| black_box(m).leading_zeros())
        });
        group.bench_with_input(BenchmarkId::new("trailing_zeros", name), &bm, |b, m| {
            b.iter(|| black_box(m).trailing_zeros())
        });
    }

    group.finish();
}

fn bench_bit_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis");

    let sparse = Bitmap256::from_value(0b101u128);
    let dense = Bitmap256::from_value(u64::MAX as u128);
    let full = !Bitmap256::new();

    for (name, bm) in [("sparse", sparse), ("dense", dense), ("full", full)] {
        group.bench_with_input(BenchmarkId::new("msb", name), &bm, |b, m| {
            b.iter(|| black_box(m).most_significant_bit())
        });
        group.bench_with_input(BenchmarkId::new("lsb", name), &bm, |b, m| {
            b.iter(|| black_box(m).least_significant_bit())
        });
    }

    group.finish();
}

fn bench_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration");

    let sparse = Bitmap256::from_value(0b101u128);
    let dense = Bitmap256::from_value(u64::MAX as u128);
    let full = !Bitmap256::new();

    for (name, bm) in [("sparse", sparse), ("dense", dense), ("full", full)] {
        group.bench_with_input(BenchmarkId::new("ones", name), &bm, |b, m| {
            b.iter(|| {
                let mut sum = 0u32;
                for idx in black_box(m).ones() {
                    sum += idx as u32;
                }
                sum
            })
        });
    }

    group.finish();
}

fn bench_shift(c: &mut Criterion) {
    let mut group = c.benchmark_group("shift");

    let bm = Bitmap256::from_value(0xABCDEF0123456789u128);

    for shift in [0, 63, 127, 128, 255] {
        group.bench_with_input(BenchmarkId::new("shl", shift), &shift, |b, &s| {
            b.iter(|| black_box(bm) << black_box(s))
        });
        group.bench_with_input(BenchmarkId::new("shr", shift), &shift, |b, &s| {
            b.iter(|| black_box(bm) >> black_box(s))
        });
    }

    group.finish();
}

fn bench_bitwise(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitwise");

    let a = Bitmap256::from_value(0xABCDEF0123456789u128);
    let b = Bitmap256::from_value(0x9876543210FEDCBAu128);

    group.bench_function("xor", |bencher| {
        bencher.iter(|| black_box(a) ^ black_box(b))
    });

    group.bench_function("not", |bencher| {
        bencher.iter(|| !black_box(a))
    });

    group.finish();
}

fn bench_has_bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup");

    let bm = Bitmap256::from_value(0xABCDEF0123456789u128);

    for bit in [0, 63, 127, 255] {
        group.bench_with_input(BenchmarkId::new("has_bit", bit), &bit, |b, &i| {
            b.iter(|| black_box(bm).has_bit(black_box(i)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_from_bit,
    bench_set_clear,
    bench_flip,
    bench_count_ones,
    bench_zeros,
    bench_bit_index,
    bench_iteration,
    bench_shift,
    bench_bitwise,
    bench_has_bit,
);
criterion_main!(benches);
