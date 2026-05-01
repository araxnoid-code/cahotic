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
