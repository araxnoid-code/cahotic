# Version/0.2.0
- opening: mekanisme packet yang digunakan `Cahotic` pada version/0.1.0 tidak memiliki nilai praktis yang tinggi saat praktek. ini dikarenakan, walaupun memiliki 64 packet dengan size masing masing packet adalah 64 yang totalnya dapat menampung 4096 task namun sering di dapatkan packet di submit dalam keadaan yang tidak penuh. Ini menyebabkan nilai praktis menurun dengan significan hingga ke titik `Cahotic` hanya dapat menampung 64 task saja di dalam kasus setiap packet hanya berisi 1 task, jika ada packet yang kosong ini akan lebih buruk. oleh karena itu dengan masalah yang menyebabkan pnurunan peforma yang signifikan ini, maka di putuskan `Cahotic` akan menggangti konsep ini.

- mengganti konsep packet(submit setiap batch) dengan konsep ring buffer dengan mekanisme antrian FIFO dalam manajemen spawn task pada `Cahotic`. mekanisme scheduling masih menggunakan konsep dulu yaitu packet(batch).

- Ukuran ring-buffer secara default saat ini adalah 4096

- menambahkan konsep quota, setiap task yang telah dibuat akan mendapakan quota serta value return task juga akan di simpan ke dalam quota yang didapatkan. untuk setiap 64 task maka akan mendapatkan quota yang sama, di dalam quota akan memiliki counter yang akan dikurangi oleh task yang telah selesai di eksekusi di saat counter menjadi 0, maka drop-bitmap diupdate berdasarkan index dari quota tersebut.

- secara default setiap quota dapat dimiliki oleh 64 task

- menghilangkan method yang berhubungan dengan konsep packet saat membuat task dan initial schedule, diantaranya adalah:
  - ready-bitmap
  - empty-bitmap
  - packet-list
  - drop-list

- menghilangkan task-core, packet-core akan mengambil tugas task-core(task-core berguna saat version/0.0.1 yang masih menggunakan konsep linked list, saat penggunaan konsep packet, task-core mulai tidak diperlukan dan hanya sebagai wrapper saja, karena itu dihilangkan).

perbandingan code versi version/0.1.0 dan version/0.2.0
version/0.1.0
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));

    cahotic.submit_packet();

    cahotic.join();
}
```

version/0.2.0
```rust
fn main() {
    // tidak diperlukan lagi initialisasi size untuk packet
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));
    
    // tidak diperlukan lagi mekanisme submit

    cahotic.join();
}
```
