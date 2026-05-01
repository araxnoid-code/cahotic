# Job

Berfungsi untuk penjadwalan.

```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultJob, DefaultOutput, Job};

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
        DefaultOutput(10)
    }))
    .after(&job_1);

    let job_3 = Job::new(DefaultJob(|_| {
        println!("3 done");
        DefaultOutput(10)
    }))
    .after(&job_2);

    cahotic.job_exec(job_1);
    cahotic.join();
}
```

penjelasan.

1. dalam pembuatan `job` menggunakan struct `Job`

```rust
let job_1 = Job::new(DefaultJob(|_| {
    sleep(Duration::from_millis(1000));
    println!("1 done");
    DefaultOutput(10)
}));
```

code diatas akan mengembalikan tipe data `Job`

2. untuk melakukan penjadwalan pada job, gunakan method `Job::after<A>(self, job: &A) -> ScheduledJob<FS, O>`

```rust
let job_2 = Job::new(DefaultJob(|_| {
    println!("2 done");
    DefaultOutput(10)
}))
.after(&job_1);
```

dengan menggunakan method `after` akan menjadwalkan `job_2` dieksekusi setelah `job_1` serta job_2 akan bertipe data `ScheduledJob`.

3. `ScheduledJob` sendiri masih bisa dijadikan sebagai parameter pada method after

```rust
let job_3 = Job::new(DefaultJob(|_| {
    println!("3 done");
    DefaultOutput(10)
}))
.after(&job_2);
```

4. dalam menjalankan job, menggunakan method `Cahotic::job_exec(&self, job: Job<FS, O>)`

```rust
cahotic.job_exec(job_1);
cahotic.join();
```

bisa dilihat hanya `job_1` saja yang dieksekusi, dikarekana `job_2` akan dan `job_3` telah dijadwalkan dan terdaftar pada `job_1` dan akan membentuk sebuh graf

```
job_1 ---> job_2 ---> job_3
```

bisa dilihat pada alur diatas, yang menjadi parameter bagi method `job_exec` hanya untuk job sebagai initial saja.

sebagai contoh lain:

```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultJob, DefaultOutput, Job};

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
        DefaultOutput(10)
    }));

    let job_3 = Job::new(DefaultJob(|_| {
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

code diatas akan membuat alur graf

```
job_1_____
          \
job_2-----------> job_3
```

bisa dilihat pada graf diatas `job_1` dan `job_2` sebagai awalan, oelh karena itu `job_1` dan `job_2` yang dijadikan parameter
untuk method `job_exec` pada bagian ini.

```rust
cahotic.job_exec(job_1);
cahotic.job_exec(job_2);
```

## Job Polling

```rust
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
```

terdapat penambahan pada code diatas, terutama pada variabel poll.
penjelasan:

```rust
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
```
mari kita lihat bagian `job_3`, `job_3` akan dieksekusi setelah `job_1` dan `job_2` selesai,
oleh karena itu `job_3` memiliki kemampuan untuk mendapatkan value return dari `job_1` dan `job_2` melalui `dependencies`.
untuk mengakses value berdasarkan urutan saat menggunakan method `after` bisa dilihat `job_1` sebagai parameter pertama lalu diikuti oleh `job_2`,
oleh karena itu index 0 digunakan untuk mengakses value return `job_1` dan index 1 untuk `job_2`

```rust
let _poll_1 = cahotic.job_exec(job_1);
let _pool_2 = cahotic.job_exec(job_2);

let poll_3 = job_3.to_poll();
let value_3 = poll_3.block();
println!("result {}", value_3.0);
```
pada baris selanjutnya, method `job_exec` akan return `PollWaiting` untuk dapat dilakukan polling dengan tujuan membaca datanya,
untuk `ScheduledJob` digunakan method `ScheduledJob::to_poll(self) -> PollWaiting<O>`, ini akan merubah `ScheduledJob` menjadi `PollWaiting`
sehingga dapat diakses valuenya dengan sistem pooling.
