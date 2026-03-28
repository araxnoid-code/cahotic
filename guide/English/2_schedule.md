# Scheduling
`schedule` functions to schedule a task and how tasks communicate with each other.
``` rust
use std::{thread::sleep, time::Duration};

use cahotic::{Cahotic, OutputTrait, ScheduleVec, SchedulerTrait, TaskTrait};

#[derive(Debug)]
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

    // Scheduling 
    let mut poll1 = cahotic.scheduling_create_initial(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("task 1 done");
        MyOutput::None
    }));

    let mut poll2 = cahotic.scheduling_create_schedule(MyTask::Schedule(|_| {
        println!("task 2 done");
        MyOutput::None
    }));

    cahotic.schedule_after(&mut poll2, &mut poll1).unwrap();

    cahotic.schedule_exec(poll2);
    cahotic.schedule_exec(poll1);

    cahotic.submit_packet();

    cahotic.join();
}
```

explanation:
```rust
let mut poll1 = cahotic.scheduling_create_initial(MyTask::Task(|| {
    sleep(Duration::from_millis(1000));
    println!("task 1 done");
    MyOutput::None
}));

let mut poll2 = cahotic.scheduling_create_schedule(MyTask::Schedule(|_| {
    println!("task 2 done");
    MyOutput::None
}));
```
Here we use 2 methods, including:
`Cahotic::scheduling_create_initial(&self, F)`
and
`Cahotic::scheduling_create_schedule(&self, FS)`
```
note:
- F: Type that implements TaskTrait (for regular tasks)
- FS: Type that implements SchedulerTrait (for scheduled tasks with dependencies)
```
To understand this, we should understand how the Scheduling concept works in `cahotic`.

## `cahotic` uses the concept of `DAG (Directional Acyclic Graph)`

<img width="400" src="./../img/DAG.png"/>

As can be seen in the image above, there are blue and red nodes. The blue node is the starting node, and the red node is the node that has been scheduled with the blue node or the red node. Based on the red and blue nodes, then:
`Cahotic::scheduling_create_initial(&self, F)`
useful for creating a initial schedule of an unschedulable graph.
`Cahotic::scheduling_create_schedule(&self, FS)`
useful for creating a normal schedule that can be scheduled by the initial schedule or normal schedule (red node).

therefore the first rule of scheduling on `cahotic`:
*every initial schedule (blue node) cannot have dependencies (it can only be a dependency of another schedule).*

Every relation created in a task must follow the `DAG` concept, namely there must be no cycles in the graph.
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    let mut poll1 = cahotic.scheduling_create_initial(MyTask::Task(|| {
        sleep(Duration::from_millis(1000));
        println!("task 1 done");
        MyOutput::None
    }));

    let mut poll2 = cahotic.scheduling_create_schedule(MyTask::Schedule(|_| {
        println!("task 2 done");
        MyOutput::None
    }));

    cahotic.schedule_after(&mut poll2, &mut poll1).unwrap();

    let mut poll3 = cahotic.scheduling_create_schedule(MyTask::Schedule(|_| {
        println!("task 3 done");
        MyOutput::None
    }));

    // a cycle occurs here, causing this task `cahotic` to get stuck
    cahotic.schedule_after(&mut poll3, &mut poll2).unwrap();
    cahotic.schedule_after(&mut poll2, &mut poll3).unwrap();

    cahotic.schedule_exec(poll3);
    cahotic.schedule_exec(poll2);
    cahotic.schedule_exec(poll1);

    cahotic.submit_packet();

    cahotic.join();
}
```
can be seen in this line
```rust
cahotic.schedule_after(&mut poll3, &mut poll2).unwrap();
cahotic.schedule_after(&mut poll2, &mut poll3).unwrap();
```
The occurrence of a cycle here causes this task to get stuck in `cahotic`.

<img width="400" src="./../img/ERROR_DAG.png">
    
hence the second rule of scheduling on `cahotic`:
*There cannot be 2 or more schedules that schedule each other (form a cycle).*

in the code below
```rust
cahotic.schedule_exec(poll3);
cahotic.schedule_exec(poll2);
cahotic.schedule_exec(poll1);

cahotic.submit_packet();

cahotic.join();
```
All initial schedules and normal schedules that have been created must be executed via the method `Cahotic::schedule_exec(&self, Schedule)` which will return `Pool Waiting`. Technically, what happens when scheduling is that the entire schedule will update the `cahotic` based on the schedule that has been set, if there are 3 schedules but only 2 are executed. then the schedule that is not executed causes the schedule to get stuck in `cahotic` and even worse, the `cahotic` cleanup mechanism gets stuck.

But if there are 3 schedules created, but none of them are executed? That's still a problem because in this method
`Cahotic::scheduling_create_schedule(&self, FS)`
directly allocate space in `cahotic`, if no schedule is executed then no schedule handles the handler to reallocate the occupied space, in other words the schedule will be stuck.

therefore the third rule of scheduling is `cahotic`: 
*All schedules that have been created must be executed.*

## Communication Between Schedules
let's look at this line:
```rust
impl SchedulerTrait<MyOutput> for MyTask {
    fn execute(&self, scheduler_vec: ScheduleVec<MyOutput>) -> MyOutput {
        match self {
            MyTask::Task(_) => MyOutput::None,
            MyTask::Schedule(f) => f(scheduler_vec),
        }
    }
}
```
there is a struct `ScheduleVec<MyOutput>`, This will accommodate all values returned by the dependent schedule.
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

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

    cahotic.submit_packet();

    cahotic.join();
}
```
pada bagian baris ini
```rust
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
```
the need for explicit determination of structure, this is the 4th rule of scheduling:
*To access the return value from the value schedule that is a dependency, it must be in accordance with the scheduling sequence.*

additionally, on this line:
```rust
cahotic.schedule_exec(poll3);
cahotic.schedule_exec(poll2);
cahotic.schedule_exec(poll1);
```
The execution order does not actually matter for random, but if it is sequential, where the deepest scheduling is executed first to the top (based on the graph formed), then it will be more optimal because there is no error handling when the schedule has finished but the schedule that depends on it has not yet entered the thread pool.

This will be the 5th rule, not to avoid errors, but for optimization.
*In scheduling execution, the deepest scheduling is executed first to the top (based on the graph formed), so it will be more optimal.*

## limitations due to design and capacity
`cahotic` will allocate space for normal schedules, due to the system design and storage limit of 64 schedules. Therefore, when more than 64 schedules are spawned at a time, `cahotic` will get stuck.

then this becomes the 6th rule.
*do not create more than 64 schedules at one time*

If there is a normal schedule that is executed without any dependencies, then the schedule can still be executed but there is a cost to handle it, it is better to use the initial schedule.
```rust
fn main() {
    let cahotic = Cahotic::<MyTask, MyTask, MyOutput, 8, 16>::init();

    // poll will still be executed but has a cost for handling, use prefix schedule.
    let poll = cahotic.scheduling_create_schedule(MyTask::Schedule(|_| {
        println!("task done");
        MyOutput::None
    }));

    cahotic.schedule_exec(poll);
    cahotic.submit_packet();

    cahotic.join();
}
```
therefore, this becomes the 7th and 8th rule
7. *every scheduling must begin with an initial schedule*
8. *avoid creating a normal schedule that is created without dependencies*

## 8 Rules, enough? I agree
So in Scheduling on `cahotic` there are a total of 8 rules, including:
1. *every initial schedule (blue node) cannot have dependencies (it can only be a dependency of another schedule).*
2. *There cannot be 2 or more schedules that schedule each other (form a cycle).*
3. *All schedules that have been created must be executed.*
4. *To access the return value from the value schedule that is a dependency, it must be in accordance with the scheduling sequence.*
5. *In scheduling execution, the deepest scheduling is executed first to the top (based on the graph formed), so it will be more optimal.*
6. *do not create more than 64 schedules at one time*
7. *every scheduling must begin with an initial schedule*
8. *avoid creating a normal schedule that is created without dependencies*
If not followed, it will result in cases that cannot yet be handled by cahotic. Therefore, caution is requested.
