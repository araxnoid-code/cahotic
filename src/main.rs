use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultJob, DefaultOutput, DependenciesVec, Job};

fn main() {
    let cahotic = CahoticBuilder::default::<usize>().build().unwrap();

    let job_1 = Job::new(DefaultJob(|_| {
        sleep(Duration::from_millis(1000));
        println!("1 done");
        DefaultOutput(10)
    }));

    let job_2 = Job::new(DefaultJob(|_| {
        sleep(Duration::from_millis(250));
        println!("2 done");
        DefaultOutput(20)
    }));

    let job_3 = Job::new(DefaultJob(
        |dependencies: DependenciesVec<DefaultOutput<usize>>| {
            println!("3 done");
            let data_1 = dependencies.get(0).unwrap().0;
            let data_2 = dependencies.get(1).unwrap().0;

            DefaultOutput(data_1 + data_2)
        },
    ))
    .after(&job_1)
    .after(&job_2);

    let _poll_1 = cahotic.job_exec(job_1);
    let _pool_2 = cahotic.job_exec(job_2);

    let poll_3 = job_3.to_poll();
    let value_3 = poll_3.block();
    println!("result {}", value_3.0);

    cahotic.join();
}
