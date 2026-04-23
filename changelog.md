# Changelog
# Version/0.3.0
- Added `CahoticBuilder` for cahotic initialization which allows fast initialization with default values and clear changes.
1. customization ring buffer size and workers
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<Option<i32>>()
        .set_ring_buffer_size::<4096>()
        .set_workers::<8>()
        .build()
        .unwrap();

    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("Done");
        DefaultOutput(None)
    }));

    cahotic.join();
}
```

2. customization Task and Shcedule
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, ScheduleVec, SchedulerTrait, TaskTrait};

// Task
struct MyTask {
    f: fn(usize) -> DefaultOutput<usize>,
    input: usize,
}

impl TaskTrait<DefaultOutput<usize>> for MyTask {
    fn execute(&self) -> DefaultOutput<usize> {
        (self.f)(self.input)
    }
}

// Schedule
struct MySchedule(fn(scheduler_vec: ScheduleVec<DefaultOutput<usize>>) -> DefaultOutput<usize>);

impl SchedulerTrait<DefaultOutput<usize>> for MySchedule {
    fn execute(&self, scheduler_vec: ScheduleVec<DefaultOutput<usize>>) -> DefaultOutput<usize> {
        (self.0)(scheduler_vec)
    }
}

fn main() {
    let cahotic = CahoticBuilder::default()
        .set_ring_buffer_size::<4096>()
        .set_workers::<8>()
        // customization Task
        .set_task_type::<MyTask>()
        // customization Schedule
        .set_schedule_type::<MySchedule>()
        .build()
        .unwrap();

    // Task
    for i in 0..5 {
        cahotic.spawn_task(MyTask {
            f: |i| {
                sleep(Duration::from_millis(1000));
                println!("task {} done", i);
                DefaultOutput(0)
            },
            input: i,
        });
    }

    // Schedule
    let mut poll_1 = cahotic.scheduling_create_initial(MyTask {
        f: |i| {
            sleep(Duration::from_millis(1000));
            println!("schedule {} done", i);
            DefaultOutput(0)
        },
        input: 1,
    });

    let mut poll_2 = cahotic.scheduling_create_schedule(MySchedule(|shcedule_vec| {
        println!("schedule 2 done");
        DefaultOutput(10)
    }));

    cahotic.schedule_after(&mut poll_2, &mut poll_1).unwrap();

    cahotic.schedule_exec(poll_2);
    cahotic.schedule_exec(poll_1);

    cahotic.join();
}
```

3. customization Output
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultSchedule, DefaultTask, OutputTrait};

struct MyOutput(&'static str);
impl OutputTrait for MyOutput {}

fn main() {
    let cahotic = CahoticBuilder::default::<i32>()
        .set_type::<DefaultTask<MyOutput>, DefaultSchedule<MyOutput>, MyOutput>()
        .build()
        .unwrap();

    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("Done");
        MyOutput("ok")
    }));

    cahotic.join();
}
```


- Added `Cahotic::try_spawn_task(&self, F)` method to spawn tasks which will return Err(TryEnqueueStatus) if the ring_buffer is full, this will not cause blocking like `Cahotic::spawn_task(&self, F)`.

- update guide to match the changes made in version 0.3.0

# Version/0.2.1
- resolves race condition bugs when the dependent schedule is still not set but the dependent schedule has finished executing.
- add testing code.

## Version/0.2.0
- opening: The packet mechanism used by `Cahotic` in version/0.1.0 does not have much practical value in practice. This is because, even though it has 64 packets with each packet size being 64, which in total can accommodate 4096 tasks, it is often found that packets are submitted in a state that is not full. This causes performance to decrease significantly to the point where `Cahotic` can only accommodate 64 tasks in the case that each packet only contains 1 task, if there are empty packets this will be even worse. Therefore, due to the problems that caused this sharp decline in performance, it was decided that `Cahotic` would replace this concept.

- replacing the packet (submit every batch) concept with a ring-buffer concept with a FIFO queue mechanism in spawn task management on `Cahotic`. The scheduling mechanism still uses the old concept, namely packet (batch).

- The current default ring-buffer size is 4096, spawning a task when the ring-buffer is still full will cause blocking on the main thread.

- adding the concept of quota, each task that has been created will get a quota and the task return value will also be saved into the quota obtained. for every 64 tasks you will get the same quota, Inside the quota there will be a counter that will be reduced by the task that has finished executing when the counter becomes 0, then the drop-bitmap is updated based on the index of the quota.

- By default, each quota can be owned by 64 tasks..

- eliminate methods related to the packet concept when creating tasks and initial schedules, including:
  - ready-bitmap
  - empty-bitmap
  - packet-list
  - drop-list

- update guide.md to reflect changes made to version/0.2.0.

- remove task-core, packet-core will take over the task-core (task-core was useful when version/0.0.1 still used the linked list concept, when using the packet concept, task-core started to be unnecessary and was only a wrapper, therefore it was removed).

comparison of code versions version/0.1.0 and version/0.2.0

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
    // no longer required size initialization for package
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));
    
    // no longer required submit mechanism
    cahotic.join();
}
```


## Version/0.1.0
This update is dominated by changes to the internal system of cahotic, especially in the scheduling and drop-packet execution mechanisms.
1. Updated the `Scheduling` mechanism.
- Added `schedule-list`. `schedule-list` is a 64-sized array that will store schedules that have not yet been executed.
- Added `allo-schedule-bitmap`. as a marker and signal for `packet-core` to store the schedule that has been created.
- Addition of `poll-schedule-bitmap`. as a marker and signal for threads in the thread pool to execute schedules that are ready to be executed.
2. Updated the `drop` mechanism
- addition of `drop-bitmap`, as a place to provide signals and positions to threads in the `thread-pool` of packets that are ready to be dropped.
3. Idle-handling
- When a thread has no tasks for some time, it will take a short `rest` to give the CPU a breather.
4. removes the old concept of requiring a local queue for each thread for drop-packet handling and scheduler.
5. added English and Indonesian language guides.

## Version/0.0.2
### 1. Packet Concept
In the previous version, the spawned task would be inserted into the swap list and then the representative thread would perform a swap to take all the tasks in the swap list to the primary list to be processed by the thread in the thread pool, but there were several shortcomings:
1. conflict between threads, each thread will compete for one part of the list, using Compare And Swap will give large latency.
2. Memory problems, too much and too much memory allocated to support the use of linked-lists in swap lists and primary lists.

Due to these shortcomings, chaotic changed the system to Packet, packet is just a name, in fact packet is a combination of several concepts from:
1. Batch, cahotic will provide a list of packets (currently the default is 64 packets) where each packet can accommodate a group of tasks into a thread pool for massive processing.
3. Bitmap, using bitmap as a mapping on packet, threads in the thread pool do not need to be constrained to one side and can go to the other side if one side is no longer possible to process, in mapping it will use bitwise scanning and bitwise updates to search for empty slots in O(1), replacing slow linear searches.
2. tail and head, in the packet there are 2 components in positioning, namely tail and head, head is needed for the main thread in entering tasks and tail functions as a place for atomic operations for the threads, inspired by the ring-buffer concept but the consumer and producer areas are separated.
4. Lock-Free, thread will not use compare_and_swap in the task fetching stage on packet, now only use fetch_add which provides security from conflicts.
5. Each packet must be submitted to be processed by the thread. If a packet is full when a task is submitted, it will automatically submit it first and search for another packet for that task.

### 2. Removal of core mechanism version/0.0.1
Since the swap-list and primary-list concepts are no longer used, the mechanisms to support the needs of swap-lists and primary-lists have been largely removed and changed, including:
1. drop-arena
2. representative-thread
3. swap-list
4. primary-list
5. logika running pada `ThreadUnit`

### 3. Inisialisasi Cahotic
on initialization:
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();
}
```
requires 5 generic types, including:
```
Cahotic::<impl TaskTrait, impl SchedulerTrait, impl OutputTrait, total thread, capacity of each packet>::init();
```

### 4. Changes to Drop Arena
The drop-arena concept has now been replaced with drop-packet, a first-come, first-served drop-packet mechanism. With a mechanism that gets a tail + 1 == packet_len value, the packet index will be listed by the thread that receives it into the packet_drop_queue to be dropped when all tasks in the packet are completed.


### 5. Changes to scheduling
scheduling now has to explicitly initialize it before it is executed by cahotic,

## Version/0.0.1
### 1. use of linked-list in swap-list and primary-list
- swap-list, functions to accommodate all tasks spawned by the main thread, tasks in this list will not be processed by threads in the thread pool.
- The primary-list serves as a place for tasks to be processed by the thread pool. This primary-list is the result of swapping in the swap-list.
### 2. Representative Thread
A representative thread is a thread that represents another thread that will perform swapping when the primary list is empty, this is to avoid conflicts between threads.
### 3. scheduler
Tasks that require output results from other tasks can use schedules to schedule their execution.
### 4. drop-arena
drop-arena will create an arena that will store all tasks spawned before `Cahotic::drop_arena(&self)` is called. When called, `list_core` will create a task wrapper that will wrap the tasks to be cleaned up and immediately send them to the `swap list`.
