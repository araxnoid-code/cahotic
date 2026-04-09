# Version/0.2.0
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
