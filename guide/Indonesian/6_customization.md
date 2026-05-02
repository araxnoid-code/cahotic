# Customization
cahotic telah menyediakan DefaultOutput, DefaultTask dan DefaultJob untuk pengembangan cepat. namun jika terdapat kasus yang membutuhkan kustomisasi maka cahotic masih menyediakannya seperti pada versi awal (version/0.2.1 ke bawah), namun pada versi terbaru telah menambahkan CahoticBuilder yang meningkatkan keterbacaan, oleh karena itu disarankan menggunakan `CahoticBuilder`.

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
ini adalah fitur default.
1. untuk output menggunakan `DefaultOutput`
2. untuk task menggunakan `DefaultTask`
3. untuk scheduling menggunakan `DefaultJob`
4. untuk size ring buffer adalah 4096
5. untuk jumlah workers adalah 4

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
untuk kustomisasi size dari ring buffer, bisa menggunakan method
`CahoticBuilder::set_ring_buffer_size<const MAX: usize>(self)`
menerima const generic, di const generic inilah size di kustomisasi. ada 2 aturan dalam mentepakan size untuk ring buffer:
1. size harus kelipatan 64
2. size tidak boleh <= 0

untuk kustomisasi jumlah workers, bisa menggunakan method
`CahoticBuilder::set_workers<const W: usize>(self)`
menerima const generic, di const generic inilah jumlah workers di kustomisasi.

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
untuk kustomisasi task, sebuah type data yang akan di jadikan task harus mengimplementasikan trait `TaskTrait` serta di daftarkan ke dalam CahoticBuilder menggunakan method `CahoticBuilder::set_task_type<T>(self)`.

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
untuk kustomisasi job, sebuah type data yang akan di jadikan job harus mengimplementasikan trait `JobTrait` serta di daftarkan ke dalam CahoticBuilder menggunakan method `CahoticBuilder::set_schedule_type<T>(self)`.

## customize Output
dalam kustomisasi output, diperlukan juga notasi untuk task dan schedule.
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
untuk kustomisasi output, sebuah type data yang akan di jadikan output harus mengimplementasikan trait `OutputTrait` serta digunakan method 
`CahoticBuilder::set_type<TASK, JOB, OUTPUT>(self)`.
