use std::hint::black_box;

use audio_gen::{context::AudioContext, generator::SampleGenerator, graph::Graph};
use criterion::{Criterion, criterion_group, criterion_main};

const BATCH_SIZE: usize = 512; // This is the batch size used in production
fn run_generator(generator: &mut SampleGenerator, num_batches: u32) {
    for _ in 0..num_batches {
        generator.batch_poll();
    }
}

fn bench_single_chain(c: &mut Criterion) {
    const N: usize = 1 << 11;
    let mut graph = Graph::new(false);
    let mut current_node_id = graph.float_node(1.);

    for _ in 0..N {
        current_node_id = graph.absolute_value_node(current_node_id);
    }

    let schedule = graph.nodes();
    let num_nodes = schedule.len();
    let mut generator =
        SampleGenerator::new(schedule, AudioContext::new(44100.), BATCH_SIZE).unwrap();

    let mut group = c.benchmark_group("single chain");
    for num_batches in [1u32, 10, 20, 100] {
        group.bench_function(
            format!(
                "{} nodes ({} batches of {} = {} samples)",
                num_nodes,
                num_batches,
                BATCH_SIZE,
                num_batches * BATCH_SIZE as u32
            )
            .as_str(),
            |b| b.iter(|| run_generator(&mut generator, black_box(num_batches))),
        );
    }
}

fn bench_binary(c: &mut Criterion) {
    const N: usize = 1 << 13;
    let mut graph = Graph::new(false);
    let mut current_node_id = graph.float_node(1.);

    for _ in 0..N {
        let float_id = graph.float_node(1.);
        current_node_id = graph.sum_node(current_node_id, float_id);
    }

    let schedule = graph.nodes();
    let num_nodes = schedule.len();
    let mut generator =
        SampleGenerator::new(schedule, AudioContext::new(44100.), BATCH_SIZE).unwrap();

    let mut group = c.benchmark_group("binary");
    for num_batches in [1u32, 10, 20, 100] {
        group.bench_function(
            format!(
                "{} nodes ({} batches of {} = {} samples)",
                num_nodes,
                num_batches,
                BATCH_SIZE,
                num_batches * BATCH_SIZE as u32
            )
            .as_str(),
            |b| b.iter(|| run_generator(&mut generator, black_box(num_batches))),
        );
    }
}

criterion_group!(benches, bench_single_chain, bench_binary);
criterion_main!(benches);
