# Schedule Execution
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    let mut poll1 = cahotic.scheduling_create_initial(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("task 1 done");
        MyOutput::Result(10)
    }));

    let mut poll2 = cahotic.scheduling_create_initial(MyTask::Task(|| {
        sleep(Duration::from_millis(500));
        println!("task 2 done");
        MyOutput::Result(20)
    }));

    // for poll3 to access the value of poll1 and the value of poll2. poll3 must first depend on poll1 and poll2
    let mut poll3 = cahotic.scheduling_create_schedule(MyTask::Schedule(|schedule_vec| {
        // in accessing the index, based on the scheduling order with poll1 and poll2
        let value_1 = schedule_vec.get(0);
        let value_2 = schedule_vec.get(1);
        println!(
            "task 3 done, value1: {:?} and value: {:?}",
            value_1, value_2
        );
        MyOutput::None
    }));

    // scheduling order will affect the index accessing poll1 and poll2 by poll3
    cahotic.schedule_after(&mut poll3, &mut poll1).unwrap(); // index 0
    cahotic.schedule_after(&mut poll3, &mut poll2).unwrap(); // index 1

    cahotic.schedule_exec(poll3);
    cahotic.schedule_exec(poll2);
    cahotic.schedule_exec(poll1);

    cahotic.join();
}
```
From the code above it can be concluded:
1. initial schedule are poll1 and poll2.
2. schedule poll3 depends on poll1 and poll2.

<img width="400" src="./../img/schedule_img_1.png">

begins by creating an initial schedule which is created using `Cahotic::scheduling_create_initial(&self, F)`
```rust
let mut poll1 = cahotic.scheduling_create_initial(MyTask::Task(|| {
    sleep(Duration::from_millis(1000));
    println!("task 1 done");
    MyOutput::Result(10)
}));

let mut poll2 = cahotic.scheduling_create_initial(MyTask::Task(|| {
    sleep(Duration::from_millis(500));
    println!("task 2 done");
    MyOutput::Result(20)
}));
```
`poll1` and `poll2` will have a value of type `Schedule<F, FS, O>`
```
note:
- F: Type that implements TaskTrait (for regular tasks)
- FS: Type that implements SchedulerTrait (for scheduled tasks with dependencies)
- O: Type that implements OutputTrait (return value of tasks)
```
When creating the initial schedule, no changes occur to cahotic. Here, we only create the `Schedule` data type, which will first collect all interactions and then be executed by `cahotic` via:
```rust
cahotic.schedule_exec(poll2); // → directly into the ring buffer
cahotic.schedule_exec(poll1); // → directly into the ring buffer
```

for schedule (normal schedule), make it via `Cahotic::scheduling_create_schedule(&self, FS)`
```rust
let mut poll3 = cahotic.scheduling_create_schedule(MyTask::Schedule(|schedule_vec| {
    // in accessing the index, based on the scheduling order with poll1 and poll2
    let value_1 = schedule_vec.get(0);
    let value_2 = schedule_vec.get(1);
    println!(
        "task 3 done, value1: {:?} and value: {:?}",
        value_1, value_2
    );
    MyOutput::None
}));
```
the above code will immediately allocate space inside the `schedule list` which is waiting inside until it is ready to be executed, The `schedule list` itself can accommodate 64 schedules at one time. If the `schedule_list` is full, the `packet-core` will experience blocking until there is free space in the `schedule_list`. `packet-core` allocates schedules on `schedule_list` using `allo-schedule-bitmap`.

The threads in the thread pool also periodically perform a quick check to see if there are any scheduled tasks ready for execution using `poll-schedule-bitmap`.

```rust
cahotic.schedule_after(&mut poll3, &mut poll1).unwrap();
cahotic.schedule_after(&mut poll3, &mut poll2).unwrap();
```
In this line, poll3 will be executed when poll1 and poll2 have finished executing. Technically poll3 has a `poll_counter` which will increase as poll3 is scheduled, and poll1 and poll2 will also store poll3's `poll_counter`. 

```rust
cahotic.schedule_exec(poll3); // → enter the schedule list, waiting for the counter
cahotic.schedule_exec(poll2); // → directly into the ring buffer
cahotic.schedule_exec(poll1); // → directly into the ring buffer
```
In the line above, all schedules created must be executed. The mechanism is in accordance with the explanation in [2_schedule.md](2_schedule.md).
for initial schedule and normal schedule have different handling when executed, initial schedule is still the same as normal task but differs only in counter handling after execution, but normal schedule is actually sent directly to `schedule_list` not to `ring-buffer`, but ring-buffer still calculates it and assumes there is a task there, This is needed for the drop schedule later, which will also be dropped along with other tasks in the quota.

In the code example above, poll3 has a `poll_counter` of 2 and when poll2 is finished, the thread executing poll2 will reduce `poll_counter` and the thread completing poll1 will also reduce `poll_counter` so that `poll_counter` is worth 0. at that time the thread that gets `poll_counter` == 0 will activate the bitmap on `poll-schedule-bitmap` according to the index occupied by the poll3 schedule.

`poll-schedule-bitmap` will be periodically checked quickly by the threads on the poll thread, still using the same concept as the drop mechanism on drop-bitmap, the thread that gets the signal from `poll-schedule-bitmap` will execute the schedule on `schedule_list` and then after that will update `allo-schedule-bitmap` to be allocated by `packet-core` for the new schedule.

<img width="400" src="./../img/schedule_0.png">
    
penjelasan:
1. normal schedule and initial schedule are created and then executed into `packet-core`
2. `packet-core` will process the schedule and place it into the `ring-buffer` and `schedule-list` in `packet-core`. for normal schedule it is still assumed to be in the ring-buffer but physically it is in the `schedule-list`.
3. The threads in the thread pool will execute the task and then when the thread finds the `poll_counter`, the thread will immediately update the `poll-schedule-bitmap`. The thread will also periodically check the `poll-schedule-bitmap` to execute the schedule that is ready to be executed.
