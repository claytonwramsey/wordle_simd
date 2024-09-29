use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use wordle::{grade, word_from_str, Grade};

pub fn bench_grade1(c: &mut Criterion) {
    let mut g = c.benchmark_group("grade1");
    g.measurement_time(Duration::from_secs(1));
    let words = [
        "roses", "hores", "eppee", "blobo", "gleeg", "dwarf", "wefts", "lowes", "horse",
    ]
    .map(|w| word_from_str(w.as_bytes()).unwrap());

    let horse = word_from_str("horse".as_bytes()).unwrap();

    g.bench_function("grade1 green", |b| {
        b.iter(|| grade(black_box(horse), black_box(horse)))
    });
    g.bench_function("grade1 black", |b| {
        b.iter(|| {
            grade(
                black_box(word_from_str("abcde".as_bytes()).unwrap()),
                black_box(word_from_str("xyzwv".as_bytes()).unwrap()),
            )
        })
    });
    g.bench_function("grade1 mixed", |b| {
        b.iter(|| grade(word_from_str("roses".as_bytes()).unwrap(), black_box(horse)));
    });

    g.bench_function("grade1 many", |b| {
        b.iter(|| {
            words
                .iter()
                .map(|&w| black_box(grade(black_box(w), black_box(horse))))
                .sum::<Grade>()
        })
    });
}

criterion_group!(grade1, bench_grade1);
criterion_main!(grade1);
