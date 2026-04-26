use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultSchedule, DefaultTask, Job};

fn main() {
    let cahotic = CahoticBuilder::default()
        .set_workers::<8>()
        .build()
        .unwrap();

    // let job_1 = Job::create_job(DefaultSchedule(|_| {
    //     sleep(Duration::from_millis(1000));
    //     println!("1 done!");
    //     DefaultOutput(10)
    // }));

    // let job_2 = Job::create_job(DefaultSchedule(|_| {
    //     sleep(Duration::from_millis(250));
    //     println!("2 done!");
    //     DefaultOutput(10)
    // }));

    // let job_3 = Job::create_job(DefaultSchedule(|_| {
    //     println!("3 done!");
    //     DefaultOutput(10)
    // }));

    // job_3.after(&job_1);
    // job_3.after(&job_2);

    // cahotic.job_exec(job_1);
    // cahotic.job_exec(job_2);

    cahotic.join();
}
