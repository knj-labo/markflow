use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use markflow_core::{get_event_iterator, MarkdownStream};
use std::io::{self, Write};

// A dummy writer that discards data, similar to /dev/null
struct NullWriter;
impl Write for NullWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn generate_large_markdown() -> String {
    let mut s = String::with_capacity(100_000);
    s.push_str("# Benchmark Document\n\n");
    for i in 0..10_000 {
        s.push_str(&format!("* List item number {}\n", i));
        s.push_str("  * Nested item with **bold** text\n");
    }
    s
}

fn benchmark_pipeline(c: &mut Criterion) {
    let input = generate_large_markdown();
    let mut group = c.benchmark_group("pipeline_throughput");

    // Calculate throughput in bytes per second
    group.throughput(Throughput::Bytes(input.len() as u64));

    // 1. Benchmark the Streaming Adapter (No intermediate String allocation)
    group.bench_function("streaming_adapter", |b| {
        b.iter(|| {
            let events = get_event_iterator(black_box(&input));
            let writer = NullWriter;
            let _ = events.stream_to_writer(writer).unwrap();
        })
    });

    // 2. Benchmark Buffering (The traditional way: Render to String, then Write)
    group.bench_function("buffering_string", |b| {
        b.iter(|| {
            let events = get_event_iterator(black_box(&input));
            let mut html_output = String::new();
            // allocating a huge string
            pulldown_cmark::html::push_html(&mut html_output, events);
            // then writing it
            let mut writer = NullWriter;
            writer.write_all(html_output.as_bytes()).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_pipeline);
criterion_main!(benches);
