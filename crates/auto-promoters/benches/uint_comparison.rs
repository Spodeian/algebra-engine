use algebra_engine::uint::Uint;
use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use num::{BigUint, One};

fn bench_addition_matrix(c: &mut Criterion) {
    let mut group = c.benchmark_group("Addition_Matrix");

    // We use a value safely within Machine limits but high enough to be "realistic"
    let m_val = usize::MAX - 100;
    let uint_m = Uint::Machine(m_val); // Fixed: previously m_m was out of scope
    let big_m = BigUint::from(m_val);

    // 1. Machine vs Raw usize (Overhead of the Enum Branch)
    group.bench_function("Machine_vs_usize/Primitive", |b| {
        b.iter(|| black_box(m_val).wrapping_add(black_box(1)))
    });
    group.bench_function("Machine_vs_usize/Uint", |b| {
        b.iter(|| black_box(uint_m.clone()) + black_box(1usize))
    });

    // 2. Promoted vs Raw BigUint (Overhead of the Enum Wrapper)
    // We create a value exactly one above usize::MAX
    let large_b = BigUint::from(usize::MAX) + BigUint::one();
    let large_u = Uint::Promoted(large_b.clone());

    group.bench_function("Promoted_vs_BigUint/BigUint", |b| {
        b.iter(|| black_box(large_b.clone()) + black_box(1u32))
    });
    group.bench_function("Promoted_vs_BigUint/Uint", |b| {
        b.iter(|| black_box(large_u.clone()) + black_box(1usize))
    });

    group.finish();
}

fn bench_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Transitions");

    // 3. The "Promotion" Event
    // Measures the logic path and the heap allocation cost when crossing the boundary
    group.bench_function("Trigger_Promotion", |b| {
        b.iter_batched(
            || Uint::Machine(usize::MAX),
            |u| black_box(u) + black_box(1usize),
            BatchSize::SmallInput,
        )
    });

    // 4. The "Demotion" Event
    // Measures the return to stack-allocated space
    group.bench_function("Trigger_Demotion", |b| {
        b.iter_batched(
            || Uint::Promoted(BigUint::from(usize::MAX) + 1u32),
            |u| black_box(u) - black_box(1usize),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, bench_addition_matrix, bench_transitions);
criterion_main!(benches);
