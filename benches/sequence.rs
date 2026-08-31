/// Benches for sequence node. The node has non-trivial logic whose performance
/// should be kept in check.
use audio_gen::{
    context::AudioContext, generator::SampleGenerator, graph::Graph, node::SourceInterval,
};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use rand::{Rng, SeedableRng, rngs::StdRng};

fn run_generator(generator: &mut SampleGenerator, num_batches: u32) {
    for _ in 0..num_batches {
        generator.batch_poll();
    }
}

pub fn bench_random_intervals(c: &mut Criterion) {
    const LAST_SAMPLE_INDEX: u32 = 10000;
    const MAX_INTERVAL_LENGTH: u32 = 100;
    const MAX_FREQUENCY: u32 = 1000;
    const NUM_SOURCES: u32 = 1000;
    const NUM_INTERVALS_PER_SOURCE: u32 = 10;
    const BATCH_SIZE: usize = 512;

    let build_schedule = || {
        let mut rng = StdRng::seed_from_u64(67);
        let mut graph = Graph::new(false);
        let mut intervals: Vec<SourceInterval> = vec![];
        for _ in 0..NUM_SOURCES {
            let frequency_id = graph.float_node(rng.random_range(1..=MAX_FREQUENCY) as f32);
            let osc_id = graph.sine_oscillator_node(frequency_id);

            for _ in 0..NUM_INTERVALS_PER_SOURCE {
                let start_idx = rng.random_range(0..(LAST_SAMPLE_INDEX - MAX_INTERVAL_LENGTH));
                let interval_length = rng.random_range(1..=MAX_INTERVAL_LENGTH);
                intervals.push(SourceInterval::new(
                    osc_id,
                    start_idx,
                    start_idx + interval_length,
                ));
            }
        }

        graph.sequence_node(intervals);
        graph.nodes()
    };
    let mut bench_group = c.benchmark_group("big sequence");
    for num_batches in [1u32, 10, 50] {
        bench_group.bench_function(
            format!(
                "{} intervals ({} batches of {} samples)",
                NUM_INTERVALS_PER_SOURCE * NUM_SOURCES,
                num_batches,
                BATCH_SIZE
            ),
            |b| {
                b.iter_batched(
                    build_schedule,
                    |schedule| {
                        let mut generator =
                            SampleGenerator::new(schedule, AudioContext::new(44100.), BATCH_SIZE)
                                .unwrap();
                        run_generator(&mut generator, num_batches)
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
}

criterion_group!(benches, bench_random_intervals);
criterion_main!(benches);
