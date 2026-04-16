# Initial
initialisasi cahotic dapat langsung melalui `Cahotic` langsung menggunakan `Cahotic::init()`, namun lebih direkomendasikan menggunakan `CahoticBuilder`
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<i32>().build().unwrap();

    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("Done");
        DefaultOutput(10)
    }));

    cahotic.join();
}
```
menggunakan `CahoticBuilder::default()` akan memberikan inisialisasi default, diantaranya:
```rust
// Output
pub struct DefaultOutput<O>(pub O)
where
    O: 'static;

// Task
pub struct DefaultTask<O>(pub fn() -> O)
where
    O: OutputTrait + 'static;

// Schedule
pub struct DefaultSchedule<O>(pub fn(vector: ScheduleVec<O>) -> O)
where
    O: OutputTrait + 'static + Send;
```
dengan menggunakan default, size ring buffer akan disediakan sebesar `4096` task dan workers sebanyak `4`.
menggunakan `CahoticBuilder` memungkinkan untuk mengatur size ring buffer, workers, type output, task dan schedule, namun untuk saat ini akan menggunakan default terlebih dahulu.


# Spawn
Di `cahotic`, ia memiliki kemampuan untuk membuat tugas yang akan dieksekusi, dengan cara:
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<i32>().build().unwrap();

    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        DefaultOutput(10)
    }));

    cahotic.join();
}
```
the explanation:
```rust
let cahotic = CahoticBuilder::default::<i32>().build().unwrap();
```
Di sini, `cahotic` dibuild menggunakan `CahoticBuilder`, disini akan menggunakan nilai default terlebih dahulu tanpa ada costumisasi.
`DefaultOutput` memerlukan notasi tipe data, notasinya dapat melalui `CahoticBuilder::default()` seperti pada code diatas.

```rust
cahotic.spawn_task(DefaultTask(|| {
    sleep(Duration::from_millis(1000));
    println!("done!");
    DefaultOutput(10)
}));
```
Pada kode di atas, menggunakan method `Cahotic::spawn_task(&self, F)`, di mana `F` adalah tipe data yang mengimplementasikan `TaskTrait`. Tugas akan tidur selama 1 detik, kemudian mencetak "done!" dan mengembalikan `DefaultOutput(10)`. method `Cahotic::spawn_task(&self, F)` ini akan mengembalikan `PollWaiting`.

`DefaultTask` di code atas merupakan type data Task standart yang bisa langsung dipakai.

sebagai akhir dari `Cahotic`, gunakan:
```rust
cahotic.join();
```
Dengan memanggil `Cahotic::join(self)`, maka ini adalah akhir dari `cahotic` dan blocking akan terjadi di sini, karena `cahotic` akan memastikan bahwa semua tugas telah selesai dan semua sampah telah dibersihkan.

# Get Poll value
Untuk dapat mengambil nilai dari polling, Anda dapat menggunakan 2 metode yang disediakan.
```rust
fn main() {
    let cahotic = CahoticBuilder::default::<i32>().build().unwrap();

    let poll = cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        DefaultOutput(10)
    }));

    // Akan ada pemblokiran pada thread utama hingga polling selesai
    let value = poll.block();
    println!("{:?}", value.0);

    cahotic.join();
}

```
or use
```rust
fn main() {
    let cahotic = CahoticBuilder::default::<i32>().build().unwrap();

    let poll = cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        DefaultOutput(10)
    }));

    // Tidak akan terjadi pemblokiran, tetapi jika polling belum siap, maka akan mengembalikan Option::None
    if let Some(value) = poll.get() {
        println!("{:?}", value.0);
    }

    cahotic.join();
}
```
