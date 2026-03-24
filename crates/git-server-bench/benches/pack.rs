use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use git_server_bench::fixtures;
use git_server_core::pack::{UploadPackRequest, generate_pack};
use std::path::Path;
use tokio::io::AsyncReadExt;

fn bench_pack_generation(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("pack_generation");

    for (name, repo_path) in [
        ("small", fixtures::small_repo()),
        ("medium", fixtures::medium_repo()),
        ("large", fixtures::large_repo()),
    ] {
        let head = get_head_oid(repo_path);
        let request = UploadPackRequest {
            wants: vec![head],
            haves: vec![],
            done: true,
        };

        group.bench_with_input(BenchmarkId::new("clone", name), &request, |b, req| {
            b.iter(|| {
                rt.block_on(async {
                    let reader = generate_pack(repo_path, req).unwrap();
                    let mut buf = Vec::new();
                    tokio::io::BufReader::new(reader)
                        .read_to_end(&mut buf)
                        .await
                        .unwrap();
                    buf
                })
            });
        });
    }

    group.finish();
}

fn get_head_oid(repo_path: &Path) -> gix::ObjectId {
    let repo = gix::open(repo_path).unwrap();
    repo.head_id().unwrap().detach()
}

criterion_group!(benches, bench_pack_generation);
criterion_main!(benches);
