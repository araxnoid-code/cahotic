# Quota
Setiap task yang di spawn akan memiliki quota, quota berfungsi sebagai pengelompokan(batching) task yang berfungsi untuk pembersihan.
pada version/0.2.0 ini jumlah quota saat ini berjumlah 64 buah dan satu quota dapat menampung 64 task.

<img width="400" src="./../img/quota_0.png">
    
dapat dilihat pada gambar diatas, setiap task akan memiliki quotanya masing-masing serta setiap 64 task akan memiliki quota yang sama.

<img width="400" src="./../img/quota_1.png">
    
pada gambar diatas adalah mekanisme counter pada quota, setiap task yang selesai akan mengurangi counter sebanyak 1, di saat task yang menjadikan counter quota menjadi 0 maka thread yang mengeksekusi task tersebut akan mengupdate drop-bitmap seseuai dengan index dari quota tersebut, dengan diupdatenya drop-bitmap ini menandakan bahwa semua value return task akan di hapus.

disaat task di spawn namun semua quota telah penuh, maka packet-core akan menunggu quota yang kosong, maka ini akan menyebabkan block pada main thread. mekanisme packet-core dapat menemukan quota yang kosong adalah melalui quota-bitmap.
