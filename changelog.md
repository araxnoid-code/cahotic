# Changelog
## Version/0.0.1
### 1. penggunaan linked-list dalam swap-list dan primary-list
- swap-list, berfungsi untuk menampung semua task yang di spawn oleh main thread, task di list ini tidak akan diproses oleh thread pada thread pool.
- primary-list, berfungsi sebagai tempat task-task yang akan di proses oleh thread pool. primary-list ini hasil dari swap pada swap-list.
### 2. Representative Thread
representative thread merupakan thread yang mewakiki thread lainnya yang akan melakukan swapping saat primary-list kosong, ini untuk menghindari konflik antar thread.
### 3. scheduler.
terhubung atau tugas yang memerlukan hasil output dati task lain, dapat menggunakan schedule untuk menjadwalkan eksekusinya.
### 4. drop-arena
drop-arena akan membuat sebuah arena yang akan menyimpan seluruh task yang di spawn sebelum `Cahotic::drop_arena(&self)` dipanggil. saat diapnggil maka `list_core` akan membuat sebuah task wrapper yang akan membungkus task yang akan di bersihkan dan langsung mengirimnya ke `swap list`.

## Version/0.0.2
### 1. Packet Concept
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


### 2. Pengahapusan mekanisme inti version/0.0.1
dikarenakan konsep swap-list dan primary-list tidak digunakan lagi, maka mekanisme untuk menunjang kebutuhan dari swap-list dan primary-list sebagian besar telah dihilangkan dan diubah, antara lain:
1. drop-arena
2. representative-thread
3. swap-list
4. primary-list
5. logika running pada `ThreadUnit`

### 2. Inisialisasi Cahotic
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

### 3. Perubahan Pada Drop Arena
konsep drop-arena kini diganti menjadi drop-packet, mekanisme drop-packet berdasarkan siapa cepat dia dapat. dengan mekanisme yang mendapatkan nilai tail + 1 == packet_len, maka index packet tersebut akan di list oleh thread yang mendapatkannya ke dalam packet_drop_queue untuk di drop di saat seluruh task di dalam packet sudah selesai.

### 4. Perubahan pada scheduling
scheduling kini harus secara eksplisit menginisialisikannya terlebih dahulu sebelum di eksekusi oleh cahotic.
