# Drop
Drop/pembersihan pada cahotic dilakukan secara batch pada satu quota yang sama.

<img width="400" src="./../img/drop_0.png">
    
1. Diawali saat ada quota yang kosong, thread akan menemukannya menggunakan memlalui drop-bitmap. mekanisme pengambilan quota disini adalah siapa cepat dia dapat.
2. Thread yang mendapatkan tugas untuk membersihkan quota akan langung membersihkannya.
3. Setelah thread selesai membersihkan quota, thread tersebut akan langsung update quota-bitmap berdasarkan index quota yang telah di bersihkan sebelumnya.
4. Packet-core akan menggunakan quota-packet untuk mendapatkan quota kosong yang siap untuk menampung task-task yang di spawn.
