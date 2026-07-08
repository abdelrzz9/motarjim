use motarjim_profiling::ProfilingSession;

fn main() {
    let mut session = ProfilingSession::new("example");

    let mut timer = session.start_phase("work");
    let mut sum = 0u64;
    for i in 0..1_000_000 {
        sum = sum.wrapping_add(i);
    }
    let elapsed = timer.stop();

    session.record_phase("work", elapsed);
    session.increment_counter("iterations", 1_000_000);

    println!("{}", session.report());
    println!("Sum: {sum}");
}
