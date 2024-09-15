use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sion::dem_tile::DemTile;
use sion::grayscale_bitmap::GrayscaleBitmap;
use sion::hillshading::igor_hillshading_opt1::hillshade as hillshade_opt1;
use sion::hillshading::igor_hillshading_opt2::hillshade as hillshade_opt2;
use sion::hillshading::igor_hillshading_orig::hillshade as hillshade_orig;
use sion::hillshading::parameters::HillshadingParameters;

fn benchmark_igor_hillshade_orig(c: &mut Criterion) {
    let dem = DemTile::from_file("tests/data/N46E006.hgt");
    let mut bitmap = GrayscaleBitmap::new(dem.size as u16, dem.size as u16);
    let parameters = HillshadingParameters::default();

    c.bench_function("igor_hillshade_orig", |b| {
        b.iter(|| {
            hillshade_orig(
                black_box(&dem),
                black_box(&parameters),
                black_box(&mut bitmap),
            )
        })
    });
}

fn benchmark_igor_hillshade_opt1(c: &mut Criterion) {
    let dem = DemTile::from_file("tests/data/N46E006.hgt");
    let mut bitmap = GrayscaleBitmap::new(dem.size as u16, dem.size as u16);
    let parameters = HillshadingParameters::default();

    c.bench_function("igor_hillshade_opt1", |b| {
        b.iter(|| {
            hillshade_opt1(
                black_box(&dem),
                black_box(&parameters),
                black_box(&mut bitmap),
            )
        })
    });
}

fn benchmark_igor_hillshade_opt2(c: &mut Criterion) {
    let dem = DemTile::from_file("tests/data/N46E006.hgt");
    let mut bitmap = GrayscaleBitmap::new(dem.size as u16, dem.size as u16);
    let parameters = HillshadingParameters::default();

    c.bench_function("igor_hillshade_opt2", |b| {
        b.iter(|| {
            hillshade_opt2(
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
    targets = benchmark_igor_hillshade_orig, benchmark_igor_hillshade_opt1, benchmark_igor_hillshade_opt2
}

criterion_main!(benches);
