<div align="center">
    <h1>cahotic</h1>
    <b><p>Thread Pool Management</p></b>
    <p>⚙️ under development ⚙️</p>
    <b>
        <p>Version / 0.0.2</p>
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
cahotic = "0.0.2"
```

### Code
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
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    // spawn task
    // memasukkan task ke dalam packet
    let poll = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));

    // submit packet
    cahotic.submit_packet();

    cahotic.join();
}
```

## Update 0.0.2
### Packet Concept
pada versi sebelumnya, task yang spawn akan di sisipkan ke dalam swap list lalu representative thread akan melakukan swap untuk mengambil semua task yang ada di swap list ke primary list untuk di proses oleh thread dalam thread pool, namun terdapat beberapa kekurangan:
1. konflik antar thread, setiap thread akan berebut pada satu bagian pada list, penggunaan Compare And Swap akan memberikan latensi besar.
2. Masalah dalam memory, terlalu banyak dan tersebarnya memory yang dialokasikan untuk menunjang penggunaan linked-list pada swap list dan primary list.
3. latensi yang cukup besar di saat banyak task yang di spawn.

atas beberapa kekurangan itu, cahotic mengganti system menjadi Packet, packet hanyalah sebutan saja, sejatinya packet adalah gabungan beberapa konsep dari:
1. Batch, cahotic akan menyediakan list packet(untuk saat ini default adalah 64 packet) yang mana tiap packet dapat menampung sekumpulan task ke dalam thread pool untuk diproses secara masif.
3. Bitmap, penggunaan bitmap sebagai pemetaan pada packet, thread pada thread pool tidak perlu harus terkekang pada satu sisi dan dapat ke sisi lain jika satu sisi sudah tidak memungkinkan untuk di proses, dalam pemetaan akan menggunakan bitwise scanning dan bitwise update untuk pencarian slot kosong secara O(1), menggantikan pencarian linear yang lambat.
2. tail and head, didalam packet terdapat 2 komponen dalam pemposisian yaitu tail dan head, head diperlukan untuk main thread dalam memasukkan task dan tail berfungsi sebagai tempat operasi atomic untuk para thread, terinspirasi dari konsep ring-buffer namun wilayah konsumer dan produsen di pisah.
4. Lock-Free, thread tidak akan menggunakan compare_and_swap dalam tahapan mengambil task pada packet, kini hanya menggunakan fetch_add yang memberikan kemanan dari konflik dan sinkronisasi tingakt cpu langsung.
5. Setiap packet haruslah disubmit untuk dapat diproses oleh thread. jika packet telah penuh saat ingin memasukkan task, maka akan secara otomatis di submit terlebih dahulu dan mencari packet lain untuk task tersebut.
```rust
// ...
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    // spawn task
    // memasukkan task ke dalam packet
    let poll = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));

    // submit packet
    cahotic.submit_packet();

    cahotic.join();
}
```

### Inisialisasi Cahotic
pada inisialisasi:
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();
}
```
memerlukan 5 generic type antara lain:
```
Cahotic::<impl TaskTrait, impl SchedulerTrait, impl OutputTrait, total thread, kapasitas setiap packet>::init();
```

### Perubahan Pada Drop Arena
konsep drop-arena kini diganti menjadi drop-packet, mekanisme drop-packet berdasarkan siapa cepat dia dapat. dengan mekanisme yang mendapatkan nilai tail + 1 == packet_len, maka index packet tersebut akan di list oleh thread yang mendapatkannya ke dalam packet_drop_queue untuk di drop di saat seluruh task di dalam packet sudah selesai.


### Perubahan pada scheduling
scheduling kini harus secara eksplisit menginisialisikannya terlebih dahulu sebelum di eksekusi oleh cahotic,
ubah terlebih dahulu:
```rust
#[derive(Debug)] // new
enum MyOutput {
    Result(i32),
    None,
}
// ...
```
scheduling:
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    // spawn task normal
    let poll = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));
    cahotic.submit_packet();

    // scheduling
    // Schedule::create_task(F)
    // berfungsi sebagai awalan dari scheduling
    // note: Schedule yang bertipe task akan mengakibatkan error saat menggunakan method Schedule::after(&self, after:Schedule)
    let mut poll1 = Schedule::create_task(MyTask::Task(|| MyOutput::Result(10)));

    // Schedule::create_schedule(FS)
    // berfungsi untuk task yang akan di schedule eksekusinya
    let mut poll2 = Schedule::create_schedule(MyTask::Schedule(|schedule_vec| {
        // mengakses index pada `schedule_vec` berdasarkan penggunakan method after
        let poll1_value = schedule_vec.get(0).unwrap();
        println!("{:?}", poll1_value);
        MyOutput::None
    }));

    // menjadwalkan poll2 dieksekusi setelah poll1 selesai
    // dikarenakan penggunaan method `after` oleh poll2 disini adalah pertama kali, maka index dari poll1 dalah 0 pada `shcedule_vec`
    poll2.after(&mut poll1).unwrap();

    // memasukkan schedule ke dalam packet
    cahotic.schedule_exec(poll1);
    cahotic.schedule_exec(poll2);

    // submit
    cahotic.submit_packet();

    cahotic.join();
}
```
