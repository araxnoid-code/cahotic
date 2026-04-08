# Spawn
Di `cahotic`, ia memiliki kemampuan untuk membuat tugas yang akan dieksekusi, dengan cara:
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

enum MyOutput {
    Result(i32),
    None,
}
impl OutputTrait for MyOutput {}

enum MyTask {
    Task(fn() -> MyOutput),
    Schedule(fn(scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput),
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
    fn execute(&self, scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Task(_) => MyOutput::None,
            MyTask::Schedule(f) => f(scheduler_vec),
        }
    }
}

fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));

    cahotic.join();
}
```
the explanation:
```rust
enum MyOutput {
    Result(i32),
    None,
}
impl OutputTrait for MyOutput {}
```
Di sini, `cahotic` memerlukan tipe data yang mengimplementasikan trait `OutputTrait` untuk dikembalikan dari tugas yang dibuat.

```rust
enum MyTask {
    Task(fn() -> MyOutput),
    Schedule(fn(scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput),
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
    fn execute(&self, scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Task(_) => MyOutput::None,
            MyTask::Schedule(f) => f(scheduler_vec),
        }
    }
}
```
Untuk spawn task di `cahotic`, `cahotic` hanya dapat menerima tipe data yang mengimplementasikan dua hal: trait `TaskTrait` dan `SchedulerTrait`. Singkatnya, `TaskTrait` digunakan untuk membuat task yang tidak memiliki dependensi atau relasi, dan `SchedulerTrait` digunakan untuk membuat task yang memiliki dependensi dan relasi dengan tugas lain (berguna untuk penjadwalan).


```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));

    cahotic.join();
}
```
Saat menginisialisasi `cahotic`, diperlukan anotasi tipe.
```rust
let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();
```
Struktur generik pada `cahotic` secara singkat:
```
Cahotic::<F, FS, O, N>::init();
Cahotic generic parameters:
- F: Tipe yang mengimplementasikan TaskTrait (untuk task reguler)
- FS: Tipe yang mengimplementasikan SchedulerTrait (untuk task terjadwal dengan dependensi)
- O: Tipe yang mengimplementasikan OutputTrait (nilai kembalian dari tugas)
- N: Jumlah thread pekerja (const generic)

note: F dan FS bisa memiliki tipe yang sama (seperti pada contoh).
```

```rust
cahotic.spawn_task(MyTask::Task(|| {
    sleep(Duration::from_millis(1000));
    println!("done!");
    MyOutput::None
}));
```
Pada kode di atas, menggunakan method `Cahotic::spawn_task(&self, F)`, di mana `F` adalah tipe data yang mengimplementasikan `TaskTrait`. Tugas akan tidur selama 1 detik, kemudian mencetak "done!" dan mengembalikan `MyOutput::None` (ingat konsep `OutputTrait` di atas). method `Cahotic::spawn_task(&self, F)` ini akan mengembalikan `PollWaiting`.


sebagai akhir dari `Cahotic`, gunakan:
```rust
cahotic.join();
```
Dengan memanggil `Cahotic::join(self)`, maka ini adalah akhir dari `cahotic` dan blocking akan terjadi di sini, karena `cahotic` akan memastikan bahwa semua tugas telah selesai dan semua sampah telah dibersihkan.

# Get Poll value
before that, let's add something
```rust
#[derive(Debug)] // Required for println! and debugging
enum MyOutput {
    Result(i32),
    None,
}
//...
```
Untuk dapat mengambil nilai dari polling, Anda dapat menggunakan 2 metode yang disediakan.
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    let poll = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::Result(10)
    }));

    // Akan ada pemblokiran pada thread utama hingga polling selesai
    let value = poll.block();
    println!("{:?}", value);

    cahotic.join();
}
```
or use
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    let poll = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::Result(10)
    }));

    // Tidak akan terjadi pemblokiran, tetapi jika polling belum siap, maka akan mengembalikan Option::None
    let value = poll.get();
    println!("{:?}", value);

    cahotic.join();
}
```
