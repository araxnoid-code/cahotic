# Job
Functions for scheduling.

```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultJob, DefaultOutput, Job};

fn main() {
    let cahotic = CahoticBuilder::default::<usize>().build().unwrap();

    let job_1 = Job::new(DefaultJob(|_| {
        sleep(Duration::from_millis(1000));
        println!("1 done");
        DefaultOutput(10)
    }));

    let job_2 = Job::new(DefaultJob(|_| {
        sleep(Duration::from_millis(250));
        println!("2 done");
        DefaultOutput(10)
    }))
    .after(&job_1);

    let job_3 = Job::new(DefaultJob(|_| {
        println!("3 done");
        DefaultOutput(10)
    }))
    .after(&job_2);

    cahotic.job_exec(job_1);
    cahotic.join();
}
```

explanation:
1. in creating a `job` using the `Job` struct

```rust
let job_1 = Job::new(DefaultJob(|_| {
    sleep(Duration::from_millis(1000));
    println!("1 done");
    DefaultOutput(10)
}));
```
The code above will return the data type `Job`

2. To schedule a job, use the method `Job::after<A>(self, job: &A) -> ScheduledJob<FS, O>`
```rust
let job_2 = Job::new(DefaultJob(|_| {
    println!("2 done");
    DefaultOutput(10)
}))
.after(&job_1);
```
by using the `after` method, `job_2` will be scheduled to be executed after `job_1` and job_2 will be of the `ScheduledJob` data type.

3. `ScheduledJob` itself can still be used as a parameter in the after method
```rust
let job_3 = Job::new(DefaultJob(|_| {
    println!("3 done");
    DefaultOutput(10)
}))
.after(&job_2);
```

4. In running a job, use the method `Cahotic::job_exec(&self, job: Job<FS, O>)`
```rust
cahotic.job_exec(job_1);
cahotic.join();
```
It can be seen that only `job_1` is executed, because `job_2` and `job_3` have been scheduled and registered with `job_1` and will form a graph.

```
job_1 ---> job_2 ---> job_3
```
can be seen in the flow above, which is a parameter for the `job_exec` method only for jobs as initials.

as another example:
```rust
use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultJob, DefaultOutput, Job};

fn main() {
    let cahotic = CahoticBuilder::default::<usize>().build().unwrap();

    let job_1 = Job::new(DefaultJob(|_| {
        sleep(Duration::from_millis(1000));
        println!("1 done");
        DefaultOutput(10)
    }));

    let job_2 = Job::new(DefaultJob(|_| {
        sleep(Duration::from_millis(250));
        println!("2 done");
        DefaultOutput(10)
    }));

    let job_3 = Job::new(DefaultJob(|_| {
        println!("3 done");
        DefaultOutput(10)
    }))
    .after(&job_1)
    .after(&job_2);

    cahotic.job_exec(job_1);
    cahotic.job_exec(job_2);
    cahotic.join();
}
```
The code above will create a graph flow.

```
job_1_____
          \
job_2-----------> job_3
```
As can be seen in the graph above, `job_1` and `job_2` are prefixes. 
Therefore, `job_1` and `job_2` are used as parameters for the `job_exec` method in this section.

```rust
cahotic.job_exec(job_1);
cahotic.job_exec(job_2);
```

## Job Polling
```rust
fn main() {
    let cahotic = CahoticBuilder::default::<usize>().build().unwrap();

    let job_1 = Job::new(DefaultJob(|_| {
        sleep(Duration::from_millis(1000));
        println!("1 done");
        DefaultOutput(10)
    }));

    let job_2 = Job::new(DefaultJob(|_| {
        sleep(Duration::from_millis(250));
        println!("2 done");
        DefaultOutput(20)
    }));

    let job_3 = Job::new(DefaultJob(
        |dependencies: DependenciesVec<DefaultOutput<usize>>| {
            println!("3 done");
            let data_1 = dependencies.get(0).unwrap().0;
            let data_2 = dependencies.get(1).unwrap().0;

            DefaultOutput(data_1 + data_2)
        },
    ))
    .after(&job_1)
    .after(&job_2);

    let _poll_1 = cahotic.job_exec(job_1);
    let _pool_2 = cahotic.job_exec(job_2);

    let poll_3 = job_3.to_poll();
    let value_3 = poll_3.block();
    println!("result {}", value_3.0);

    cahotic.join();
}
```
There are additions to the code above, especially in the poll variable.

explanation:
```rust
let job_1 = Job::new(DefaultJob(|_| {
    sleep(Duration::from_millis(1000));
    println!("1 done");
    DefaultOutput(10)
}));

let job_2 = Job::new(DefaultJob(|_| {
    sleep(Duration::from_millis(250));
    println!("2 done");
    DefaultOutput(20)
}));

let job_3 = Job::new(DefaultJob(
    |dependencies: DependenciesVec<DefaultOutput<usize>>| {
        println!("3 done");
        let data_1 = dependencies.get(0).unwrap().0;
        let data_2 = dependencies.get(1).unwrap().0;

        DefaultOutput(data_1 + data_2)
    },
))
.after(&job_1)
.after(&job_2);
```
Let's look at the `job_3` section. `job_3` will be executed after `job_1` and `job_2` are completed.
Therefore, `job_3` has the ability to get the return value from `job_1` and `job_2` through `dependencies`.
To access values in order when using the `after` method, you can see `job_1` as the first parameter, followed by `job_2`.
Therefore, index 0 is used to access the return value of `job_1` and index 1 for `job_2`.

```rust
let _poll_1 = cahotic.job_exec(job_1);
let _pool_2 = cahotic.job_exec(job_2);

let poll_3 = job_3.to_poll();
let value_3 = poll_3.block();
println!("result {}", value_3.0);
```
In the next line, the `job_exec` method will return `PollWaiting` to be able to poll for the purpose of reading the data.
`ScheduledJob` uses the `ScheduledJob::to_poll(self) -> PollWaiting<O>` method, 
which will convert `ScheduledJob` to `PollWaiting` so that its value can be accessed using the pooling system.
