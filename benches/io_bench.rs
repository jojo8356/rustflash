use criterion::{Criterion, criterion_group, criterion_main};

fn bench_hash_computation(c: &mut Criterion) {
    let data = vec![0u8; 4 * 1024 * 1024]; // 4 MiB block

    c.bench_function("blake3_4mb", |b| {
        b.iter(|| blake3::hash(&data));
    });

    c.bench_function("sha256_4mb", |b| {
        b.iter(|| {
            use sha2::{Digest, Sha256};
            Sha256::digest(&data)
        });
    });
}

fn bench_compression(c: &mut Criterion) {
    let data = vec![0u8; 4 * 1024 * 1024]; // 4 MiB block (zeros = highly compressible)

    c.bench_function("zstd_compress_4mb", |b| {
        b.iter(|| zstd::encode_all(data.as_slice(), 3).unwrap());
    });

    c.bench_function("gzip_compress_4mb", |b| {
        b.iter(|| {
            use flate2::write::GzEncoder;
            use std::io::Write;
            let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&data).unwrap();
            encoder.finish().unwrap()
        });
    });
}

criterion_group!(benches, bench_hash_computation, bench_compression);
criterion_main!(benches);
