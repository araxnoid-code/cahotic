# Customization
cahotic telah menyediakan DefaultOutput, DefaultTask dan DefaultSchedule untuk pengembangan cepat. namun jika terdapat kasus yang membutuhkan kustomisasi maka cahotic masih menyediakannya seperti pada versi awal (version/0.2.1 ke bawah), namun pada versi terbaru telah menambahkan CahoticBuilder yang meningkatkan keterbacaan, oleh karena itu disarankan menggunakan `CahoticBuilder`.

## Default
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultSchedule, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<i32>().build().unwrap();

    // task
    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("Done");
        DefaultOutput(10)
    }));

    // scheduling
    let mut poll_1 = cahotic.scheduling_create_initial(DefaultTask(|| DefaultOutput(20)));
    let mut poll_2 =
        cahotic.scheduling_create_schedule(DefaultSchedule(|schedule_vec| DefaultOutput(20)));
    cahotic.schedule_after(&mut poll_2, &mut poll_1).unwrap();

    cahotic.schedule_exec(poll_2);
    cahotic.schedule_exec(poll_1);

    cahotic.join();
}

```
ini adalah fitur default.
1. untuk output menggunakan `DefaultOutput`
2. untuk task menggunakan `DefaultTask`
3. untuk schedule menggunakan `DefaultSchedule`
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

## customize Schedule
```rust
use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask, ScheduleVec, SchedulerTrait, TaskTrait};

struct MySchedule(fn(schedule_vec: ScheduleVec<DefaultOutput<usize>>) -> DefaultOutput<usize>);

impl SchedulerTrait<DefaultOutput<usize>> for MySchedule {
    fn execute(&self, scheduler_vec: ScheduleVec<DefaultOutput<usize>>) -> DefaultOutput<usize> {
        (self.0)(scheduler_vec)
    }
}

fn main() {
    let cahotic = CahoticBuilder::default::<usize>()
        .set_schedule_type::<MySchedule>()
        .build()
        .unwrap();

    let mut poll_1 = cahotic.scheduling_create_initial(DefaultTask(|| DefaultOutput(10)));
    let mut poll_2 = cahotic.scheduling_create_initial(DefaultTask(|| DefaultOutput(20)));

    let mut poll_3 = cahotic.scheduling_create_schedule(MySchedule(|vec| {
        let data_1 = vec.get(0).unwrap();
        let data_2 = vec.get(1).unwrap();
        let result = data_1.0 + data_2.0;
        println!("result => {}", result);

        DefaultOutput(result)
    }));

    cahotic.schedule_after(&mut poll_3, &mut poll_1).unwrap();
    cahotic.schedule_after(&mut poll_3, &mut poll_2).unwrap();

    cahotic.schedule_exec(poll_3);
    cahotic.schedule_exec(poll_2);
    cahotic.schedule_exec(poll_1);

    cahotic.join();
}
```
untuk kustomisasi schedule, sebuah type data yang akan di jadikan schedule harus mengimplementasikan trait `SchedulerTrait` serta di daftarkan ke dalam CahoticBuilder menggunakan method `CahoticBuilder::set_schedule_type<T>(self)`.

## customize Output
dalam kustomisasi output, diperlukan juga notasi untuk task dan schedule.
```rust
use cahotic::{CahoticBuilder, DefaultSchedule, DefaultTask, OutputTrait};

enum MyOutput {
    Number(i32),
    Float(f64),
}

impl OutputTrait for MyOutput {}

fn main() {
    let cahotic = CahoticBuilder::default::<usize>()
        .set_type::<DefaultTask<MyOutput>, DefaultSchedule<MyOutput>, MyOutput>()
        .build()
        .unwrap();

    cahotic.spawn_task(DefaultTask(|| MyOutput::Number(100)));
    cahotic.spawn_task(DefaultTask(|| MyOutput::Float(32.0)));

    cahotic.join();
}
```
untuk kustomisasi output, sebuah type data yang akan di jadikan output harus mengimplementasikan trait `OutputTrait` serta digunakan method 
`CahoticBuilder::set_type<TASK, SCHEDULE, OUTPUT>(self)`.
