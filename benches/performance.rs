use criterion::{criterion_group, criterion_main};

use voxelhex::contree::{Albedo, Contree, V3c};

fn criterion_benchmark(c: &mut criterion::Criterion) {
    {
        let tree_size = 512;
        let mut tree: Contree = Contree::new(tree_size, 8).ok().unwrap();
        for x in 0..100 {
            for y in 0..100 {
                for z in 0..100 {
                    if x < (tree_size / 4)
                        || y < (tree_size / 4)
                        || z < (tree_size / 4)
                        || ((tree_size / 2) <= x && (tree_size / 2) <= y && (tree_size / 2) <= z)
                    {
                        tree.insert(&V3c::new(x, y, z), &Albedo::from(0x00ABCDEF))
                            .ok()
                            .unwrap();
                    }
                }
            }
        }
    }

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let tree_size = 64;
    let mut tree: Contree = Contree::new(tree_size, 8).ok().unwrap();
    for _i in 0..50000000 {
        tree.insert(
            &V3c::new(
                rng.gen_range(0..tree_size),
                rng.gen_range(0..tree_size),
                rng.gen_range(0..tree_size),
            ),
            &Albedo::from(rng.gen_range(0..50000)),
        )
        .expect("Octree insert to suceeed");
    }
    c.bench_function("contree insert", |b| {
        b.iter(|| {
            tree.insert(
                &V3c::new(
                    rng.gen_range(0..tree_size),
                    rng.gen_range(0..tree_size),
                    rng.gen_range(0..tree_size),
                ),
                &Albedo::from(rng.gen_range(0..50000)),
            )
            .ok()
        });
    });

    c.bench_function("contree clear", |b| {
        b.iter(|| {
            tree.clear(&V3c::new(
                rng.gen_range(0..tree_size),
                rng.gen_range(0..tree_size),
                rng.gen_range(0..tree_size),
            ))
            .ok()
            .unwrap();
        });
    });

    c.bench_function("contree get", |b| {
        b.iter(|| {
            tree.get(&V3c::new(
                rng.gen_range(0..tree_size),
                rng.gen_range(0..tree_size),
                rng.gen_range(0..tree_size),
            ));
        });
    });
    #[cfg(feature = "bytecode")]
    {
        c.bench_function("contree save", |b| {
            b.iter(|| {
                tree.save("test_junk_contree").ok().unwrap();
            });
        });

        c.bench_function("contree load", |b| {
            b.iter(|| {
                let _tree_copy = Contree::<Albedo>::load("test_junk_contree").ok().unwrap();
            });
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
