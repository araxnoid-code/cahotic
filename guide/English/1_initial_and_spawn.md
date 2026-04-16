# Initial
Cahotic initialization can be done directly through `Cahotic` directly using `Cahotic::init()`, but it is more recommended to use `CahoticBuilder`
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<i32>().build().unwrap();

    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("Done");
        DefaultOutput(10)
    }));

    cahotic.join();
}
```
using `CahoticBuilder::default()` will provide default initialization, including:
```rust
// Output
pub struct DefaultOutput<O>(pub O)
where
    O: 'static;

// Task
pub struct DefaultTask<O>(pub fn() -> O)
where
    O: OutputTrait + 'static;

// Schedule
pub struct DefaultSchedule<O>(pub fn(vector: ScheduleVec<O>) -> O)
where
    O: OutputTrait + 'static + Send;
```
by using default, the ring buffer size will be provided as `4096` tasks and as many as `4` workers.
Using `CahoticBuilder` allows you to set the ring buffer size, workers, output type, task and schedule, but for now we will use the defaults first.

# Spawn
In `cahotic`, it has the ability to create tasks to be executed, in a way:
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<i32>().build().unwrap();

    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        DefaultOutput(10)
    }));

    cahotic.join();
}
```
the explanation:
```rust
let cahotic = CahoticBuilder::default::<i32>().build().unwrap();
```
Here, `cahotic` is built using `CahoticBuilder`, which uses the default values without any customization.
`DefaultOutput` requires data type notation, which can be done using `CahoticBuilder::default()`, as in the code above.

```rust
cahotic.spawn_task(DefaultTask(|| {
    sleep(Duration::from_millis(1000));
    println!("done!");
    DefaultOutput(10)
}));
```
In the code above, using the `Cahotic::spawn_task(&self, F)` method, where `F` is a data type that implements `TaskTrait`. The task will sleep for 1 second, then print "done!" and return `DefaultOutput(10)`. This `Cahotic::spawn_task(&self, F)` method will return `PollWaiting`.

`DefaultTask` in the code above is a standard Task data type that can be used directly.

as the end of `Cahotic`, use:
```rust
cahotic.join();
```
By calling `Cahotic::join(self)`, this is the end of `cahotic` and blocking will occur here, because `cahotic` will ensure that all tasks have been completed and all garbage has been cleaned up.

# Get Poll value
To be able to retrieve values from polling, you can use the 2 methods provided.
```rust
fn main() {
    let cahotic = CahoticBuilder::default::<i32>().build().unwrap();

    let poll = cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        DefaultOutput(10)
    }));

    // There will be a block on the main thread until the poll is complete
    let value = poll.block();
    println!("{:?}", value.0);

    cahotic.join();
}

```
or use
```rust
fn main() {
    let cahotic = CahoticBuilder::default::<i32>().build().unwrap();

    let poll = cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("done!");
        DefaultOutput(10)
    }));

    // There will be no blocking, but if the poll is not ready, it will return Option::None
    if let Some(value) = poll.get() {
        println!("{:?}", value.0);
    }

    cahotic.join();
}
```
