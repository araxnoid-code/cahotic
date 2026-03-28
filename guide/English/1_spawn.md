# Spawn
in `cahotic`, it has the ability to spawn a task to be executed, by:
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

enum MyOutput {
    Result(i32),
    None,
}
impl OutputTrait for MyOutput {}

enum MyTask {
    Task(fn() -> MyOutput),
    Schedule(fn(scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput),
}

impl TaskTrait<MyOutput> for MyTask {
    fn execute(&self) -> MyOutput {
        match self {
            MyTask::Task(f) => f(),
            MyTask::Schedule(_) => MyOutput::None,
        }
    }
}

impl SchedulerTrait<MyOutput> for MyTask {
    fn execute(&self, scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Task(_) => MyOutput::None,
            MyTask::Schedule(f) => f(scheduler_vec),
        }
    }
}

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
the explanation:
```rust
enum MyOutput {
    Result(i32),
    None,
}
impl OutputTrait for MyOutput {}
```
Here, `cahotic` requires a data type that implements the `OutputTrait` trait to be returned from the spawned task.

```rust
enum MyTask {
    Task(fn() -> MyOutput),
    Schedule(fn(scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput),
}

impl TaskTrait<MyOutput> for MyTask {
    fn execute(&self) -> MyOutput {
        match self {
            MyTask::Task(f) => f(),
            MyTask::Schedule(_) => MyOutput::None,
        }
    }
}

impl SchedulerTrait<MyOutput> for MyTask {
    fn execute(&self, scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Task(_) => MyOutput::None,
            MyTask::Schedule(f) => f(scheduler_vec),
        }
    }
}
```
To spawn tasks in `cahotic`, `cahotic` can only accept data types that implement two things: the `TaskTrait` and `SchedulerTrait` traits. In short, `TaskTrait` is used to spawn tasks that have no dependencies or relationships, and `SchedulerTrait` is used to spawn tasks that have dependencies and relationships with other tasks (useful for scheduling).


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
when initializing `cahotic`, requires explicit type annotation.
```rust
let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();
```
generic structure on `cahotic` in brief:
```
Cahotic::<F, FS, O, N, PN>::init();
Cahotic generic parameters:
- F: Type that implements TaskTrait (for regular tasks)
- FS: Type that implements SchedulerTrait (for scheduled tasks with dependencies)
- O: Type that implements OutputTrait (return value of tasks)
- N: Number of worker threads (const generic)
- PN: Packet capacity — maximum tasks per packet (const generic)

Note: F and FS can be the same type (as in the example).
```
*What is a `packet`?*

Cahotic groups tasks into batches called `packet` (default 64 packets, each with capacity PN). Packets will be explained in detail in the next section.

```rust
cahotic.spawn_task(MyTask::Task(|| {
    sleep(Duration::from_millis(1000));
    println!("done!");
    MyOutput::None
}));
```
In the code above, using the `Cahotic::spawn_task(&self, F)` method, where `F` is a data type that implements `TaskTrait`. The task will sleep for 1 second, then print "done!" and return `MyOuput::None` (remember the `OutputTrait` concept above). This method `Cahotic::spawn_task(&self, F)` will return `PollWaiting`

```rust
cahotic.submit_packet();
```
The spawned task has not actually been sent to the thread pool, but is still waiting in a storage area called `packet`, by using the `Cahotic::submit_packet(&self)` method, the packet will be sent to the thread pool and the thread will start executing it.

```rust
cahotic.join();
```
by calling `Cahotic::join(self)`, then this is the end of `cahotic` and blocking will occur here, because `cahotic` will ensure that all tasks have been completed and all garbage has been removed.

# Get Poll value
before that, let's add something
```rust
#[derive(Debug)] // Required for println! and debugging
enum MyOutput {
    Result(i32),
    None,
}
//...
```
To be able to retrieve values ​​from the poll, you can use the 2 methods provided
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    let poll = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::Result(10)
    }));
    cahotic.submit_packet();

    // there will be a block on the main thread until the poll is complete
    let value = poll.block();
    println!("{:?}", value);

    cahotic.join();
}
```
or use
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    let poll = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::Result(10)
    }));
    cahotic.submit_packet();

    // no block will occur, but if the poll is not ready yet, it will return Option::None
    let value = poll.get();
    println!("{:?}", value);

    cahotic.join();
}
```
