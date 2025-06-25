use ark_bls12_381::{Fr, G1Projective};
use ark_ff::{One, Zero};
use ark_std::UniformRand;
use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, BenchmarkId, Criterion, criterion_group, criterion_main};
use lvc::commit::interpolate_and_commit;
use lvc::lvc::{LagrangeLvc, Lvc, Proof};
use lvc::setup::Setup;
use rand::seq::index::sample;
use rand::{Rng, thread_rng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn bench_setup(n: &usize, group: &mut BenchmarkGroup<WallTime>) {
    group.bench_function(BenchmarkId::new("setup", format!("n: {}", n)), |bencher| {
        bencher.iter(|| Setup::build(*n))
    });
}

fn bench_commit(setup: &Setup, a: &[Fr], group: &mut BenchmarkGroup<WallTime>) {
    group.bench_function(
        BenchmarkId::new("commit", format!("n: {}", a.len())),
        |bencher| {
            bencher.iter(|| interpolate_and_commit::<G1Projective>(setup.domain, &setup.pk_g1, a))
        },
    );
}

fn bench_open(setup: &Setup, a: &[Fr], b: &[Fr], group: &mut BenchmarkGroup<WallTime>) {
    let b_one = b.par_iter().filter(|v| **v == Fr::one()).count();
    group.bench_function(
        BenchmarkId::new("open", format!("n: {} - b: {}", a.len(), b_one)),
        |bencher| bencher.iter(|| LagrangeLvc::open(setup, a, b)),
    );
}

fn bench_verify(
    setup: &Setup,
    commit: &G1Projective,
    b: &[Fr],
    proof: &Proof,
    group: &mut BenchmarkGroup<WallTime>,
) {
    group.bench_function(
        BenchmarkId::new("verify", format!("n: {}", b.len())),
        |bencher| {
            bencher.iter(|| {
                let _ = LagrangeLvc::verify(setup, commit, b, proof);
            })
        },
    );
}

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("lvc");

    let n = vec![
        2_i32.pow(14),
        2_i32.pow(18),
        2_i32.pow(22),
        2_i32.pow(24),
        2_i32.pow(26),
    ];

    for n in n {
        let n = n as usize;
        let a: Vec<Fr> = (0..n).map(|_| Fr::rand(&mut thread_rng())).collect();

        let mut b = vec![Fr::zero(); n];
        let b_size = thread_rng().gen_range(1..n);

        let indices = sample(&mut thread_rng(), n, b_size);
        for i in indices.iter() {
            b[i] = Fr::one();
        }

        bench_setup(&n, &mut group);
        let setup = Setup::build(n).expect("Setup generation failed.");

        bench_commit(&setup, &a, &mut group);
        let commit = interpolate_and_commit(setup.domain, &setup.pk_g1, &a)
            .expect("Commit computation failed.");

        bench_open(&setup, &a, &b, &mut group);
        let proof = LagrangeLvc::open(&setup, &a, &b).expect("Failed to compute proof.");

        bench_verify(&setup, &commit, &b, &proof, &mut group);
        let check = LagrangeLvc::verify(&setup, &commit, &b, &proof);
        assert!(check.is_ok(), "Verification failed.");
    }

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
