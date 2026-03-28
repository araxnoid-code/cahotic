# Packet
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("1 done!");
        MyOutput::None
    }));

    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        println!("2 done!");
        MyOutput::None
    }));

    cahotic.submit_packet();

    cahotic.join();
}
```
In the code above, cahotic will not immediately send the task that has been spawned into the thread pool, but will first put it into a `packet`.

<img width="400" src="./../img/packet_submit.png">
    
explanation:
1. task created first
```rust
MyTask::Task(|| {
    sleep(Duration::from_millis(1000));
    println!("1 done!");
    MyOutput::None
}
```
2. using the method `Cahotic::spawn_task(&self, F)`. to make it `WaitingTask`.
```rust
cahotic.spawn_task(MyTask::Task(|| {
    sleep(Duration::from_millis(1000));
    println!("1 done!");
    MyOutput::None
}));
```
3. Waiting Tasks are not directly executed into the thread pool, but must wait and be collected into packets, so the thread pool will receive tasks in batches.
4. using the method `Cahotic::submit_packet(&self)`
```rust
cahotic.submit_packet();
```
5. When submitting, it does not actually send a packet to the thread pool, but will update the `ready-bitmap`, `ready-bitmap` is a bitmap that functions as a mapping and as a status provider, the use of bitmaps is used because they are light but can scan quickly.
6. when `ready-bitmap` is updated from 0 to 1, the thread pool can execute it.

When submitting, the main thread will immediately look for an empty packet to be used as a container for the tasks that will come next.

<img width="400" src="./../img/packet_after_submit.png">
    
In the image above, we replace `main pool` with `packet-core`. Overall, it's the same as the explanation above, but technically, the name for the `cahotic` that handles all packet administration is called `packet-core`.
the explanation
1. `packet-core` does not point to any packets during initialization. Therefore, `packet-core` will search for empty packets using `empty-bitmap`.
2.  When `packet-core` finds an empty packet, that packet will accommodate the incoming tasks.
3.  packet submitted
4. `packet-core` will update ready-bitmap.
5. `thread pool` will check the bitmap and get information about the location and status of the packet.
6. `packet-core` now has no destination packet, therefore `packet-core` will look for an empty packet.
7.  Once an empty packet is found, `packet-core` will make it a container for future tasks.

`packet-core` has 64 packets, each of which has a capacity that can be set using the initialization notation `Cahotic`
```
Cahotic::<F, FS, O, N, PN>::init();
Cahotic generic parameters:
- F: Type that implements TaskTrait (for regular tasks)
- FS: Type that implements SchedulerTrait (for scheduled tasks with dependencies)
- O: Type that implements OutputTrait (return value of tasks)
- N: Number of worker threads (const generic)
- PN: Packet capacity — maximum tasks per packet (const generic)
```
Based on the initialization structure code above, the capacity of the packet can be set via `PN`

note!: there are some special conditions of this `packet-core`
1. If the packet is full and a task comes in, then `packet-core` will automatically submit the packet and look for an empty packet.
2. If `packet-core` searches for empty packets, but finds all packets are full, then blocking will occur here until `packet-core` finds an empty packet.

as additional information, packets are stored in an array of 64, therefore `empty-bimtap` and `ready-bimtap` will be 64 bits in size which is useful not only for getting status but also for getting position quickly.
example
```
[empty, empty, ready, empty, ready, ready]
empty-bimtap: 110100
ready-bimtap: 001011
```

There's a special case for scheduling. The initial schedule will still be included in the packet, but the normal schedule will be considered included in the packet, but only for cleaning purposes. Technically, the schedule will be included in the schedule-list. We'll discuss this later.
