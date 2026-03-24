use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use git_server_bench::fixtures;
use git_server_core::discovery::RepoStore;
use std::net::SocketAddr;
use std::path::Path;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

fn bench_http_clone(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("http_clone");

    for (name, repo_path) in [
        ("small", fixtures::small_repo()),
        ("medium", fixtures::medium_repo()),
        ("large", fixtures::large_repo()),
    ] {
        let (addr, shutdown_tx, handle) = rt.block_on(start_server(repo_path));

        group.bench_with_input(BenchmarkId::new("clone", name), &addr, |b, addr| {
            b.iter(|| {
                rt.block_on(async {
                    let client = reqwest::Client::new();
                    let base = format!("http://{}", addr);

                    // GET info/refs
                    let refs_resp = client
                        .get(format!(
                            "{}/repo.git/info/refs?service=git-upload-pack",
                            base
                        ))
                        .send()
                        .await
                        .unwrap();
                    let refs_body = refs_resp.bytes().await.unwrap();

                    let head_oid = extract_head_oid(&refs_body);
                    let request_body = build_upload_pack_request(&head_oid);

                    // POST git-upload-pack
                    let pack_resp = client
                        .post(format!("{}/repo.git/git-upload-pack", base))
                        .header("content-type", "application/x-git-upload-pack-request")
                        .body(request_body)
                        .send()
                        .await
                        .unwrap();
                    let _pack_body = pack_resp.bytes().await.unwrap();
                })
            });
        });

        let _ = shutdown_tx.send(());
        rt.block_on(async {
            let _ = handle.await;
        });
    }

    group.finish();
}

/// Extract the HEAD OID from a pkt-line encoded refs advertisement.
///
/// The advertisement format is:
///   <pkt-line># service=git-upload-pack\n
///   0000  (flush)
///   <pkt-line><oid> HEAD\0<capabilities>\n
///   ...
///   0000
///
/// We skip all pkt-lines up to and including the first flush packet, then
/// read the OID (first 40 hex characters) from the following pkt-line payload.
fn extract_head_oid(refs_body: &[u8]) -> String {
    let mut pos = 0;
    while pos + 4 <= refs_body.len() {
        let len_str = match std::str::from_utf8(&refs_body[pos..pos + 4]) {
            Ok(s) => s,
            Err(_) => break,
        };
        let pkt_len = match usize::from_str_radix(len_str, 16) {
            Ok(n) => n,
            Err(_) => break,
        };
        if pkt_len == 0 {
            // Flush packet -- the next pkt-line is the first ref line.
            pos += 4;
            if pos + 4 > refs_body.len() {
                break;
            }
            let next_str = match std::str::from_utf8(&refs_body[pos..pos + 4]) {
                Ok(s) => s,
                Err(_) => break,
            };
            let next_len = match usize::from_str_radix(next_str, 16) {
                Ok(n) => n,
                Err(_) => break,
            };
            if next_len >= 44 && pos + next_len <= refs_body.len() {
                let payload = &refs_body[pos + 4..pos + 4 + 40];
                if let Ok(oid) = std::str::from_utf8(payload) {
                    if oid.chars().all(|c| c.is_ascii_hexdigit()) {
                        return oid.to_string();
                    }
                }
            }
            pos += next_len.max(4);
        } else if pkt_len >= 4 {
            pos += pkt_len;
        } else {
            break;
        }
    }
    panic!("could not extract HEAD OID from ref advertisement");
}

fn build_upload_pack_request(oid: &str) -> Vec<u8> {
    let want_line = format!("want {oid}\n");
    let want_len = want_line.len() + 4;
    let mut body = format!("{want_len:04x}{want_line}").into_bytes();
    body.extend_from_slice(b"00000009done\n");
    body
}

async fn start_server(
    repo_path: &Path,
) -> (SocketAddr, oneshot::Sender<()>, tokio::task::JoinHandle<()>) {
    let store = RepoStore::discover(repo_path.parent().unwrap().to_path_buf(), 0).unwrap();
    let router = git_server_http::router(store);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = oneshot::channel();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = rx.await;
            })
            .await
            .unwrap();
    });
    (addr, tx, handle)
}

criterion_group!(benches, bench_http_clone);
criterion_main!(benches);
