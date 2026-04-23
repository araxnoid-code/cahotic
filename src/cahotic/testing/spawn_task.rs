use std::{thread::sleep, time::Duration};

use crate::{CahoticBuilder, DefaultOutput, DefaultTask};

#[test]
fn spawn_task() {
    let cahotic = CahoticBuilder::default().build().unwrap();

    // testing 1
    // check spawn 1 task
    cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));

    // testing 2
    // check spawn 3 task
    // // 1
    cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));
    cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));
    cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));

    // // 2
    for _ in 0..64 {
        cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));
    }

    for _ in 0..64 {
        cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));
    }

    for _ in 0..64 {
        cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));
    }

    // testing 3
    // check spawn 3 task yang delay
    // // 1
    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(500));
        DefaultOutput(0)
    }));
    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(500));
        DefaultOutput(0)
    }));
    cahotic.spawn_task(DefaultTask(|| {
        sleep(Duration::from_millis(500));
        DefaultOutput(0)
    }));

    // // 2
    for _ in 0..64 {
        cahotic.spawn_task(DefaultTask(|| {
            sleep(Duration::from_millis(100));
            DefaultOutput(0)
        }));
    }

    for _ in 0..64 {
        cahotic.spawn_task(DefaultTask(|| {
            sleep(Duration::from_millis(100));
            DefaultOutput(0)
        }));
    }

    // testing 4
    for _ in 0..64 {
        cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));
    }

    for _ in 0..64 * 8 {
        cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));
    }

    for _ in 0..64 * 16 {
        cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));
    }

    for _ in 0..64 * 32 {
        cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));
    }

    for _ in 0..64 * 64 {
        cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));
    }

    for _ in 0..64 * 128 {
        cahotic.spawn_task(DefaultTask(|| DefaultOutput(0)));
    }

    cahotic.join();
}
