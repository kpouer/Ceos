use ceos::ceos::search::SearchMatcher;
use ceos::ceos::search::simple_search_case_sensitive::SimpleSearchCaseSensitiveMatcher;
use ceos::ceos::search::simple_search_matcher_case_insensitive::SimpleSearchCaseInsensitiveMatcher;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_search(c: &mut Criterion) {
    let line = "Ceos est un visualiseur de logs haute performance écrit en Rust, conçu pour gérer facilement des fichiers de plusieurs gigaoctets.";

    let mut group = c.benchmark_group("SimpleSearchMatcher");

    let matcher = SimpleSearchCaseSensitiveMatcher::new("performance", false);
    group.bench_function("search_exact_match", |b| {
        b.iter(|| matcher.search(black_box(line), black_box(0)))
    });

    let matcher = SimpleSearchCaseInsensitiveMatcher::new("PERFORMANCE", false);
    group.bench_function("search_case_insensitive", |b| {
        b.iter(|| matcher.search(black_box(line), black_box(0)))
    });

    let matcher = SimpleSearchCaseSensitiveMatcher::new("logs", true);
    group.bench_function("search_whole_word", |b| {
        b.iter(|| matcher.search(black_box(line), black_box(0)))
    });

    let matcher = SimpleSearchCaseSensitiveMatcher::new("notfound", false);
    group.bench_function("search_no_match", |b| {
        b.iter(|| matcher.search(black_box(line), black_box(0)))
    });

    group.finish();
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
