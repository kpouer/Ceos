use ceos::ceos::search::SearchMatcher;
use ceos::ceos::search::simple_search_matcher::SimpleSearchMatcher;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_search(c: &mut Criterion) {
    let line = "Ceos est un visualiseur de logs haute performance écrit en Rust, conçu pour gérer facilement des fichiers de plusieurs gigaoctets.";

    let mut group = c.benchmark_group("SimpleSearchMatcher");

    group.bench_function("search_exact_match", |b| {
        let matcher = SimpleSearchMatcher::new("performance", true, false);
        b.iter(|| matcher.search(black_box(line), black_box(0)))
    });

    group.bench_function("search_case_insensitive", |b| {
        let matcher = SimpleSearchMatcher::new("PERFORMANCE", false, false);
        b.iter(|| matcher.search(black_box(line), black_box(0)))
    });

    group.bench_function("search_whole_word", |b| {
        let matcher = SimpleSearchMatcher::new("logs", true, true);
        b.iter(|| matcher.search(black_box(line), black_box(0)))
    });

    group.bench_function("search_no_match", |b| {
        let matcher = SimpleSearchMatcher::new("notfound", true, false);
        b.iter(|| matcher.search(black_box(line), black_box(0)))
    });

    group.finish();
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
