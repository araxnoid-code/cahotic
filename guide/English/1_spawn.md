# Spawn
In `cahotic`, it has the ability to create tasks to be executed, in a way:
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
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));

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
Here, `cahotic` requires a data type that implements the `OutputTrait` trait to be returned from the created task.
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
To spawn tasks in `cahotic`, `cahotic` can only accept data types that implement two things: the `TaskTrait` and `SchedulerTrait` traits. In short, `TaskTrait` is used to create tasks that have no dependencies or relationships, and `SchedulerTrait` is used to create tasks that have dependencies and relationships with other tasks (useful for scheduling).

```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::None
    }));

    cahotic.join();
}
```
When initializing `cahotic`, a type annotation is required.
```rust
let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();
```
The generic structure of `cahotic` in brief:
```
Cahotic::<F, FS, O, N>::init();
Cahotic generic parameters:
- F: Types that implement TaskTrait (for regular tasks)
- FS: Types that implement the SchedulerTrait (for scheduled tasks with dependencies)
- O: Types that implement OutputTrait (return value from a task)
- N: Number of worker threads (const generic)

note: F and FS can have the same type (as in the example).
```

```rust
cahotic.spawn_task(MyTask::Task(|| {
    sleep(Duration::from_millis(1000));
    println!("done!");
    MyOutput::None
}));
```
In the code above, using the `Cahotic::spawn_task(&self, F)` method, where `F` is a data type that implements `TaskTrait`. The task will sleep for 1 second, then print "done!" and return `MyOutput::None` (remember the `OutputTrait` concept above). This `Cahotic::spawn_task(&self, F)` method will return `PollWaiting`.

as the end of `Cahotic`, use:
```rust
cahotic.join();
```
By calling `Cahotic::join(self)`, this is the end of `cahotic` and blocking will occur here, because `cahotic` will ensure that all tasks have been completed and all garbage has been cleaned up.

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
To be able to retrieve values ​​from polling, you can use the 2 methods provided.
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    let poll = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::Result(10)
    }));

    // There will be a block on the main thread until the poll is complete
    let value = poll.block();
    println!("{:?}", value);

    cahotic.join();
}
```
or use
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8>::init();

    let poll = cahotic.spawn_task(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        MyOutput::Result(10)
    }));

    // There will be no blocking, but if the poll is not ready, it will return Option::None
    let value = poll.get();
    println!("{:?}", value);

    cahotic.join();
}
```
