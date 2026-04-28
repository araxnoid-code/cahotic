use cahotic::{Cahotic, CahoticBuilder, DefaultJob, DefaultOutput, DefaultTask, Job, ScheduledJob};

fn main() {
    let cahotic: Cahotic<
        DefaultTask<DefaultOutput<usize>>,
        DefaultJob<DefaultOutput<usize>>,
        DefaultOutput<usize>,
        4,
        4096,
    > = CahoticBuilder::default::<usize>().build().unwrap();

    let tensor_c: Job<DefaultJob<DefaultOutput<usize>>, DefaultOutput<usize>> =
        Job::create_job(DefaultJob(|_| {
            println!("done task {} + {} = {}", 10, 20, 10 + 20);
            DefaultOutput(10 + 20)
        }));

    let tensor_d: ScheduledJob<DefaultJob<DefaultOutput<usize>>, DefaultOutput<usize>> =
        Job::create_job(DefaultJob(|dep| {
            let c: &DefaultOutput<usize> = dep.get(0).unwrap();

            println!("done task {} + {} = {}", c.0, 20, c.0 + 20);
            DefaultOutput(c.0 + 20)
        }))
        .after(&tensor_c);

    let tensor_e: ScheduledJob<DefaultJob<DefaultOutput<usize>>, DefaultOutput<usize>> =
        Job::create_job(DefaultJob(|dep| {
            let c: &DefaultOutput<usize> = dep.get(0).unwrap();
            let d: &DefaultOutput<usize> = dep.get(1).unwrap();

            println!("done task {} + {} = {}", c.0, d.0, c.0 + d.0);
            DefaultOutput(c.0 + d.0)
        }))
        .after(&tensor_c)
        .after(&tensor_d);

    cahotic.job_exec(tensor_c);

    cahotic.join();
}
