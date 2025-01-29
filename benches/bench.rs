use criterion::{criterion_group, criterion_main, Criterion};

fn contains(line: &str, filter: &str) -> bool {
    line.contains(filter)
}

fn containssimd(line: &str, filter: &str) -> bool {
    memchr::memmem::find(line.as_bytes(), filter.as_bytes()).is_some()
}

fn containssimd2(line: &[u8], filter: &[u8]) -> bool {
    memchr::memmem::find(line, filter).is_some()
}

fn criterion_benchmark(c: &mut Criterion) {
    let line = "2021-07-27 16:28:47.679 | WARN  | Thread TCP | ?                   :      | [] Missing firmware revision AVP";
    let search = "firmware";
    let line2 = line.as_bytes();
    let search2 = search.as_bytes();
    c.bench_function("normal", |b| b.iter(|| contains(line, search)));

    c.bench_function("simd", |b| b.iter(|| containssimd(line, search)));
    c.bench_function("simd2", |b| b.iter(|| containssimd2(line2, search2)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
