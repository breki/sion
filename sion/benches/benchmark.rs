use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sion::dem_tile::DemTile;
use sion::grayscale_bitmap::GrayscaleBitmap;
use sion::hillshading::igor_hillshading::hillshade;
use sion::hillshading::parameters::HillshadingParameters;

fn benchmark_hillshade(c: &mut Criterion) {
    let dem = DemTile::from_file("tests/data/N46E006.hgt");
    let mut bitmap = GrayscaleBitmap::new(dem.size, dem.size);
    let parameters = HillshadingParameters::default();

    c.bench_function("hillshade", |b| {
        b.iter(|| {
            hillshade(
                black_box(&dem),
                black_box(&parameters),
                black_box(&mut bitmap),
            )
        })
    });
}

fn criterion_config() -> Criterion {
    Criterion::default().sample_size(10) // Set the sample count to 50
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets = benchmark_hillshade
}

criterion_main!(benches);
