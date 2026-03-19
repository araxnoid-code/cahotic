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
    cahotic.drop_arena();

    cahotic.join();
}
```

## Architecture
### Core
#### Cahotic
sebagai wrapper untuk core-core lainnya.
#### List Core
Berfungsi untuk menyiapkan setiap tugas yang dispawn dan menyisipkannya ke dalam `swap list` lalu menghasilkan `poll`,
manajement serta menghapus `arena` dan menghapus `poll`.

terdapat `swap list` sebagai list yang berfokus untuk menerima semua tugas dari main thread.
terdapat  `primary list` sebagai list yang berfokus pada konsumsi task oleh thread unit.
#### Thread Pool Core
berfungsi sebagai wadah `thread unit` berjalan, mengatur setiap kebutuhan thread unit,
manajemen share variable serta memberikan handling untuk event event tertentu.
#### Thread Unit
sebagai pekerja yang akan mengeksekusi setiap task, menghapus setiap `arena` dengan bantuan `list core`, 
chek counter scheduler serta counter arena.


## Fitur
### Thread Pool
inisialisasi thread pool, memerlukan sebuah type yang mengimplementasikan TaskTrait, SchedulerTrait dan OutputTrait serta inisialisasi jumlah thread yang akan di spawn
```rust
let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();
// Cahotic::<impl TaskTrait, impl SchedulerTrait, impl OutputTrait, total thread>::init();
// total thread tidak boleh 0
```

### Task
akan spawn sebuah task yang akan masuk ke dalam list yang menunggu untuk di eksekusi
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
    cahotic.drop_arena();

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
    cahotic.drop_arena();

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

    cahotic.drop_arena();

    cahotic.join();
}
```
pada code diatas, poll3 akan dieksekusi saat poll1 dan poll2 sudah selesai

### Drop Arena
drop arena menggunakan konsep yaitu
1. Arena Allocator
2. Dedicated Thread Cleaner/Dedicated Janitor/apalah namanya
drop-arena akan membuat sebuah arena yang akan menyimpan seluruh task yang di spawn sebelum `Cahotic::drop_arena(&self)` dipanggil.
saat diapnggil maka `list_core` akan membuat sebuah task wrapper yang akan membungkus task yang akan di bersihkan dan langsung
mengirimnya ke `swap list`.

saat thread mendapatkan task wrapper tersebut, maka thread tersebut akan memeriksa apakah setiap task yang terkandung dalam wrapper
telah selesai semua atau belum dengan cara melihat `done_counter`, jika sudah maka thread tersebut akan fokus menghapus data pada arena.
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    let poll1 = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::Result(10)
    }));

    let poll2 = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::Result(10)
    }));

    // akan membuat wrapper arena yang terkandung poll1 dan poll2
    // yang akan dihapus di saat kedua poll selesai
    cahotic.drop_arena();

    // poll1 dan poll2 berkemungkinan telah dihapus disini
    // mengakses poll setelah .drop_arena() sangat rawan

    cahotic.join();
}
```

### join
method `Cahotic::.join(self)` merupakan akhir dari Cahotic. disini akan memastikan semua task telah selesai
semua arena telah dibersihkan, semua sampah dibersihkan dan penghentian `thread unit`. 
berkemungkinan akan terjadi blocking saat menggunakannya method ini.
