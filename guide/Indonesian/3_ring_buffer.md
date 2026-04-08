# Ring-Buffer
Task yang telah di spawn akan dimasukkan dan menunggu di dalam ring-buffer untuk di eksekusi oleh thread di dalam thread pool.

<img width="400" src="./../img/ring_buffer_0.png">
    
gambar di atas adalah mekanisme task yang masuk ke dalam packet-core menjadi waiting task, waiting task sendiri akan di tempatkan ke dalam ring buffer berdasarkan nilai head yang dia dapatkan. Disaat head sudah sampai di ujung maka head akan kembali lagi pada index awal.

Pada kasus ring-buffer penuh, maka packet-core akan menunggu ring-buffer ada space untuk alokasi berikutnya, ini akan menyebabkan main thread terblock.

<img width="400" src="./../img/ring_buffer_1.png">

Pada gambar diatas menunjukkan bagaimana cara thread pada thread pool mengambil task di dalam ring-buffer. Thread akan mendapatkan index yang telah di tentukan oleh tail(menggunakan operasi atomic fetch add), setelah itu thread akan mengambil task yang pada index yang telah di dapatkan. namun di saat index yang di dapatkan melewati index head, maka thread tidak akan melakukan sinkronisasi dan akan menyimpan index tersebut dan menjadikannya "pesanan" yang akan di check secara berkala oleh thread, di saat packet-core mengisi index yang telah di pesan ini maka thread akan langsung mengambilnya tanpa harus mengambil index baru di dalam tail.
