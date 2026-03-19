<div align="center">
    <h1>cahotic</h1>
    <b><p>Thread Pool Management</p></b>
    <p>⚙️ under development ⚙️</p>
    <b>
        <p>Version / 0.0.1</p>
    </b>
</div>

## About
`cahotic`, thread pool management written in rust.

## Starting
### Installation
Run the following Cargo command in your project directory:
```sh
cargo add cahotic
```
Or add the following line to your Cargo.toml:
```toml
cahotic = "0.0.1"
```

### Code
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{Cahotic, OutputTrait, SchedulerTrait, SchedulerVec, TaskTrait};

enum MyOutput {
    Result(i32),
    None,
}
impl OutputTrait for MyOutput {}

enum MyTask {
    Task(fn() -> MyOutput),
    Schedule(fn(scheduler_vec: SchedulerVec<MyOutput>) -> MyOutput),
}

impl TaskTrait<MyOutput> for MyTask {
    fn execute(&self) -> MyOutput {
        match self {
            MyTask::Task(f) => f(),
            MyTask::Schedule(_) => MyOutput::None,
        }
    }
}

impl SchedulerTrait<MyOutput> for MyTask {
    fn execute(&self, scheduler_vec: SchedulerVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Task(_) => MyOutput::None,
            MyTask::Schedule(f) => f(scheduler_vec),
        }
    }
}

fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    // spawn task
    let poll = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));

    // membersihkan task yang telah dieksekusi
    cahotic.swap_drop_arena();

    cahotic.join();
}
```

## Fitur
### Thread Pool
inisialisasi thread pool, memerlukan sebuah type yang mengimplementasikan TaskTrait, SchedulerTrait dan OutputTrait serta inisialisasi jumlah thread yang akan di spawn
```rust
let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();
// Cahotic::<impl TaskTrait, impl SchedulerTrait, impl OutputTrait, total thread>::init();
// total thread tidak boleh 0
```

### Task
akan menernakkan sebuah task yang akan masuk ke dalam list yang menunggu untuk di eksekusi
```rust
// ...
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    // spawn task
    let poll = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));

    // membersihkan task yang telah dieksekusi
    cahotic.swap_drop_arena();

    cahotic.join();
}
```
untuk mendapatkan nilai dari task yang di spawn, dapat menggunakan
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    // spawn task
    let poll = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::Result(10)
    }));

    // akan terjadi blocking disini sehingga data siap
    poll.block();
    let result = poll.get();
    if let Some(MyOutput::Result(num)) = result {
        println!("{}", num);
    }

    // membersihkan task yang telah dieksekusi
    cahotic.swap_drop_arena();

    cahotic.join();
}
```
atau dapat langsung menggunakan
```rust
// ...
let result = poll.collect();
if let MyOutput::Result(num) = result {
    println!("{}", num);
}
// ...
```

### Scheduler
tugas yang terhubung atau tugas yang memerlukan hasil output dati task lain, dapat menggunakan schedule untuk menjadwalkan eksekusinya.
```rust
// untuk schadule, harus memastikan impl SchedulerTrait terlebih dahulu
// ini hanya contoh, untuk implementasinya bisa lebih luas
impl SchedulerTrait<MyOutput> for MyTask {
    fn execute(&self, scheduler_vec: SchedulerVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Task(_) => MyOutput::None,
            MyTask::Schedule(f) => f(scheduler_vec),
        }
    }
}
```
sebelum itu, mari kita ubah MyOutput terlebih dahulu
```rust
#[derive(Debug)]
enum MyOutput {
    Result(i32),
    None,
}
```
untuk membuat scheduler harus menggunakan struct `Scheduler`
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    // spawn task
    let poll1 = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::Result(10)
    }));

    let poll2 = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(2000));
        println!("done!");
        MyOutput::Result(20)
    }));

    // membuat shceduler
    let mut shceduler = Scheduler::init(MyTask::Schedule(|schedule_vec| {
        // urutan akses berdasarkan urutan method before!
        let poll1 = schedule_vec.get(0).unwrap();
        let poll2 = schedule_vec.get(1).unwrap();
        println!("poll 1:{:?} & poll2:{:?}", poll1, poll2);
        MyOutput::None
    }));

    // urutan method before mempengaruhi urutan pengaksesan!
    shceduler.before(&poll1);
    shceduler.before(&poll2);

    // eksekusi
    let poll3 = cahotic.scheduler_exec(shceduler);

    cahotic.swap_drop_arena();

    cahotic.join();
}
```
pada code diatas, poll3 akan dieksekusi saat poll1 dan poll2 sudah selesai

## Drop Arena
drop arena menggunakan konsep yaitu
1. Arena Allocator
2. Double Buffering
drop arena memiliki 2 arena yaitu arena0 dan arena1 dan saat pertamakali inisialisasi akan menggunakan arena0 secara default.

setiap task yang di spwan akan masuk ke dalam arena yang aktif lalu hingga digunakannya method `Cahotic::swap_drop_arena(&self)`
akan berpindah arena sekaligus mengechek apakah semua task pada arena sebelumnya seudah selesai atau belum, 
jika sudah maka akan langsung di drop/dihapus dan jika belum maka tidak terjadi perpindahan hingga `Cahotic::swap_drop_arena(&self)` digunakan kembali.
