# Version/0.4.0
In short, the changes that occurred include:
- removes the use of the `Schedule` struct
- Code that still uses the old schedule concept has been removed or changed
- scheduling now uses `Job`
- change `SchedulerTrait` to `JobTrait`
- change the flow of usage in scheduling
- execution on `Job` is different from before which used `Schedule`
- add a job ring-buffer that is the same size as the task ring-buffer
- Quota can now be accessed by tasks and jobs without interfering with each other


## opening
because scheduling in Cahotic version 0.3.1 has limitations, namely it can only create 64 schedules at one time and separate schedules in the scheduling cannot communicate with each other directly, so for uses that require large scheduling it will be very disrupted. Therefore cahotic version 0.4.0 removes `Schedule` and replaces it with `job`

## job
job, is a structure for scheduling tasks that are spawned, the concept is the same as the previous schedule structure but jobs have technical differences.

jobs do not use allo-schedule-bitmap and poll-schedule-bitmap which are limited to 64bit which means they can only store 64 schedules, jobs use the same concept as ring buffers in tasks, jobs will be called job ring buffers.

when a job is run, in contrast to the schedule which will be `considered` to be in the task ring buffer for cleaning purposes, the job will now not interfere with the task section and the `quota` mechanism for cleaning purposes can now be accessed by tasks and jobs without having to influence each other's ring buffers on tasks and jobs.

With this job there will be no problems regarding the size of scheduling at one time, but there are several drawbacks, namely blocking of the workers thread when the job ring buffer is full.

the size of the job ring buffer will be the same as the size of the task ring buffer. because the schedule has been deleted, things related to the schedule have now been removed.

Now the mechanism for spawning jobs and running jobs is slightly different in terms of flow compared to the previous schedule:
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultJob, DefaultOutput, Job};

fn main() {
    let cahotic = CahoticBuilder::default::<usize>().build().unwrap();

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

    let job_3 = Job::new(DefaultJob(|_| {
        sleep(Duration::from_millis(1000));
        println!("3 done");
        DefaultOutput(10)
    }))
    .after(&job_1)
    .after(&job_2);

    cahotic.job_exec(job_1);
    cahotic.job_exec(job_2);

    cahotic.join();
}
```
