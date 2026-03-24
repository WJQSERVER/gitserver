use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use git_server_bench::fixtures;
use git_server_core::refs::advertise_refs;

fn bench_ref_advertisement(c: &mut Criterion) {
    let mut group = c.benchmark_group("ref_advertisement");

    for (name, repo_path) in [
        ("small", fixtures::small_repo()),
        ("medium", fixtures::medium_repo()),
        ("large", fixtures::large_repo()),
    ] {
        group.bench_with_input(BenchmarkId::new("advertise", name), repo_path, |b, path| {
            b.iter(|| advertise_refs(path).unwrap());
        });
    }

    group.finish();
}

criterion_group!(benches, bench_ref_advertisement);
criterion_main!(benches);
