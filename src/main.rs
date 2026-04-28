use std::{thread::sleep, time::Duration};

use cahotic::{Cahotic, CahoticBuilder, DefaultJob, DefaultOutput, DefaultTask, Job, ScheduledJob};

fn main() {
    let cahotic: Cahotic<
        DefaultTask<DefaultOutput<usize>>,
        DefaultJob<DefaultOutput<usize>>,
        DefaultOutput<usize>,
        4,
        4096,
    > = CahoticBuilder::default::<usize>().build().unwrap();

    for i in 0..62 {
        cahotic.spawn_task(DefaultTask(|| DefaultOutput(10)));
    }

    let job_1 = Job::new(DefaultJob(|_| {
        sleep(Duration::from_millis(500));
        println!("1 done");
        DefaultOutput(10)
    }));

    let job_2 = Job::new(DefaultJob(|_| {
        sleep(Duration::from_millis(250));
        println!("2 done");
        DefaultOutput(10)
    }));

    // let job_3 = Job::new(DefaultJob(|_| {
    //     sleep(Duration::from_millis(1000));
    //     println!("3 done");
    //     DefaultOutput(10)
    // }))
    // .after(&job_1)
    // .after(&job_2);

    cahotic.job_exec(job_1);
    cahotic.job_exec(job_2);

    cahotic.join();
}
