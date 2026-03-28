# Task Execution
pada packet terdapat `task_list`, `tail` dan `head`. untuk eksekusi task, para thread hanya memerlukan `task_list` dan `tail`.
```
note:
pada task juga memiliki head, namun head berfungsi untuk drop dan packet-core.
```

<img width="400" src="./../img/task_list_packet.png">
    
alur dari thread mengambil task yaitu:
1. thread akan check packet apa yang `ready` untuk di eksekusi menggunakan `ready-bitmap`.
2. di saat bertemu packet, maka thread akan langsung mengambil satu task menggunakan tail serta operasi `fetch_add` atomic untuk menghindari konflik.
4. di saat ada task yang telah mencapai ujung list. maka task tersebut akan langsung mereset bit pada ready-bitma pada lokasi packet.
5. di saat ada satu thread yang telah melewati ujung dari list(ujung list disini adalah kapasitas maksimal list, bukan head). maka thread tersebut akan mencari packet selanjutnya melalui `ready-bitmap` kembali.

## pencarian packet akan terus maju(next-fit)
cara thread mencari tidak dari satu titik yang sama setiap saat. ini akan meningkatkan tabrakan karena banyak thread juga mencari di tempat yang sama pula serta task task pada packet ujung akan lama dieksekusi karena packet `packet-core` akan berfokus pada pencarian cepat. oleh karena itu thread mengimplementasi konsep next-fit yang mana thread akan mencari pada posisi terakhir kali dia mendapatkan packet.

<img width="400" src="./../img/next-fit-thread.png">
alur bagaimana thread mencari telah digambarkan di atas, pencarian menggunakan operasi bit secara langsung jadi tidak ada pencarian linear yang terjadi.
di saat tidak apa packet lagi, maka thread akan kembali pada pososi awal.

## operasi atomic
untuk menghindari konflik, `cahotic` menggunakan operiasi penjumlahan atomic berdasarkan dari tail, langsung mendapatkan sinkronisasi tingkat cpu dan setiap thread akan langsung mendapatkan posisi mereka karena index sendiri hanya ada satu dan unik.

<img width="400" src="./../img/atomic_add.png">
    
bisa di lihat diatas, jika banyak thread yang menjumlahkan tail secara atomic maka akan berlaku siapa cepat ia dapat dan mengakibatkan tidak berurutan, namun itu tidaklah menjadi masalah karena eksekusi task secepat mungkin adalah inti dari `cahotic`.
