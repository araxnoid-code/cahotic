use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultSchedule, DefaultTask, Job};

fn main() {
    let cahotic = CahoticBuilder::default()
        .set_workers::<16>()
        .build()
        .unwrap();

    for i in 0..62 {
        cahotic.spawn_task(DefaultTask(|| {
            sleep(Duration::from_millis(250));
            DefaultOutput(0)
        }));
    }

    let job_1 = Job::create_job(DefaultSchedule(|_| {
        sleep(Duration::from_millis(1000));
        println!("1 done!");
        DefaultOutput(10)
    }));

    let job_2 = Job::create_job(DefaultSchedule(|_| {
        sleep(Duration::from_millis(250));
        println!("2 done!");
        DefaultOutput(20)
    }));

    let job_3 = Job::create_job(DefaultSchedule(|vec| {
        sleep(Duration::from_millis(2000));
        let value_1: &DefaultOutput<i32> = vec.get(0).unwrap();
        let value_2 = vec.get(1).unwrap();
        println!("3 done! with value {} and {}", value_1.0, value_2.0);
        DefaultOutput(10)
    }));

    job_3.after(&job_1);
    job_3.after(&job_2);

    cahotic.job_exec(job_1);
    cahotic.job_exec(job_2);

    cahotic.join();
}
