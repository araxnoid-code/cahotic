# Customization
cahotic has provided DefaultOutput, DefaultTask, and DefaultJob for rapid development. However, if there are cases that require customization, cahotic still provides them as in the initial version (version/0.2.1 and below), but in the latest version has added CahoticBuilder which improves readability, therefore it is recommended to use `CahoticBuilder`.

## Default
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultJob, DefaultOutput, DefaultTask, Job};

fn main() {
    let cahotic = CahoticBuilder::default::<i32>().build().unwrap();

    // task
    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("Done");
        DefaultOutput(10)
    }));

    // scheduling
    let poll_1 = Job::new(DefaultJob(|_| DefaultOutput(20)));
    let poll_2 = Job::new(DefaultJob(|_| DefaultOutput(20))).after(&poll_1);

    cahotic.job_exec(poll_1);

    cahotic.join();
}
```
this is a default feature.
1. for output use `DefaultOutput`
2. for tasks using `DefaultTask`
3. to scheduling use `DefaultJob`
4. for the ring buffer size is 4096
5. for the number of workers is 4

## customize ring buffer size and number of workers
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<i32>()
        .set_ring_buffer_size::<2048>() // ring buffer size
        .set_workers::<8>() // jumlah workers
        .build()
        .unwrap();

    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("Done");
        DefaultOutput(10)
    }));

    cahotic.join();
}
```
To customize the size of the ring buffer, you can use the method
`CahoticBuilder::set_ring_buffer_size<const MAX: usize>(self)`
accepts const generic, in const generic this is where the size is customized. There are 2 rules for determining the size for the ring buffer:
1. size must be a multiple of 64
2. size cannot be <= 0

To customize the number of workers, you can use the method
`CahoticBuilder::set_workers<const W: usize>(self)`
accepts const generic, in this const generic the number of workers is customized.

## customize Task
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, TaskTrait};

struct MyTask {
    f: fn(usize) -> DefaultOutput<usize>,
    input: usize,
}

impl TaskTrait<DefaultOutput<usize>> for MyTask {
    fn execute(&self) -> DefaultOutput<usize> {
        (self.f)(self.input)
    }
}

fn main() {
    let cahotic = CahoticBuilder::default::<usize>()
        .set_task_type::<MyTask>() // task type
        .build()
        .unwrap();

    for i in 0..10 {
        cahotic.spawn_task(MyTask {
            f: |input| {
                sleep(Duration::from_millis(100));
                println!("task {} done", input);
                DefaultOutput(10)
            },
            input: i,
        });
    }

    cahotic.join();
}
```
To customize a task, a data type that will be used as a task must implement the `TaskTrait` trait and be registered in CahoticBuilder using the `CahoticBuilder::set_task_type<T>(self)` method.

## customize Job
```rust
use cahotic::{CahoticBuilder, DefaultOutput, DependenciesVec, Job, JobTrait};

struct MyJob(fn(schedule_vec: DependenciesVec<DefaultOutput<usize>>) -> DefaultOutput<usize>);

impl JobTrait<DefaultOutput<usize>> for MyJob {
    fn execute(
        &self,
        scheduler_vec: DependenciesVec<DefaultOutput<usize>>,
    ) -> DefaultOutput<usize> {
        (self.0)(scheduler_vec)
    }
}

fn main() {
    let cahotic = CahoticBuilder::default::<usize>()
        .set_schedule_type::<MyJob>()
        .build()
        .unwrap();

    let job_1 = Job::new(MyJob(|_| DefaultOutput(10)));
    let job_2 = Job::new(MyJob(|_| DefaultOutput(20)));

    let job_3 = Job::new(MyJob(|vec| {
        let data_1 = vec.get(0).unwrap();
        let data_2 = vec.get(1).unwrap();
        let result = data_1.0 + data_2.0;
        println!("result => {}", result);

        DefaultOutput(result)
    }))
    .after(&job_1)
    .after(&job_2);

    cahotic.job_exec(job_1);
    cahotic.job_exec(job_2);

    cahotic.join();
}
```
To customize the job, a data type that will be used as a schedule must implement the `JobTrait` trait and be registered in CahoticBuilder using the `CahoticBuilder::set_schedule_type<T>(self)` method.

## customize Output
In customizing the output, notation for tasks and schedules is also required.
```rust
use cahotic::{CahoticBuilder, DefaultJob, DefaultTask, OutputTrait};

enum MyOutput {
    Number(i32),
    Float(f64),
}

impl OutputTrait for MyOutput {}

fn main() {
    let cahotic = CahoticBuilder::default::<usize>()
        .set_type::<DefaultTask<MyOutput>, DefaultJob<MyOutput>, MyOutput>()
        .build()
        .unwrap();

    cahotic.spawn_task(DefaultTask(|| MyOutput::Number(100)));
    cahotic.spawn_task(DefaultTask(|| MyOutput::Float(32.0)));

    cahotic.join();
}
```
To customize the output, a data type that will be output must implement the `OutputTrait` trait and use the `CahoticBuilder::set_type<TASK, JOB, OUTPUT>(self)` method.
