use ceos::ceos::buffer::buffer::Buffer;
use ceos::ceos::command::search::Search;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::mpsc::channel;

fn bench_search_init(c: &mut Criterion) {
    let (sender, _) = channel();

    // Créer un buffer de test avec 100 000 lignes
    let mut content = String::with_capacity(110 * 100_000);
    for i in 0..100_000 {
        content.push_str(&format!(
            "Ceci est la ligne numéro {} avec du texte de remplissage pour simuler un log.\n",
            i
        ));
    }

    // On utilise une taille de groupe de 1000 pour la compression (si active)
    let buffer = Buffer::new_from_string(sender, &content, 1000);

    let mut group = c.benchmark_group("Search::init");

    group.bench_function("search_init_100k_lines", |b| {
        b.iter(|| {
            let mut search = Search::try_from("s numéro 50000").expect("Failed to create search");
            search.init(black_box(&buffer));
            black_box(search)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_search_init);
criterion_main!(benches);
