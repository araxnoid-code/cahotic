use std::{thread::sleep, time::Duration};

use cahotic::{CahoticBuilder, DefaultOutput, DefaultSchedule, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default().build().unwrap();

    let mut poll1 = cahotic.scheduling_create_initial(DefaultTask(|| {
        sleep(Duration::from_millis(1000));
        println!("task 1 done");
        DefaultOutput(10)
    }));

    let mut poll2 = cahotic.scheduling_create_initial(DefaultTask(|| {
        sleep(Duration::from_millis(500));
        println!("task 2 done");
        DefaultOutput(20)
    }));

    // for poll3 to access the value of poll1 and the value of poll2. poll3 must first depend on poll1 and poll2
    let mut poll3 = cahotic.scheduling_create_schedule(DefaultSchedule(|schedule_vec| {
        // in accessing the index, based on the scheduling order with poll1 and poll2
        let value_1 = schedule_vec.get(0).unwrap();
        let value_2 = schedule_vec.get(1).unwrap();
        println!(
            "task 3 done, value1: {:?} and value: {:?}",
            value_1.0, value_2.0
        );
        DefaultOutput(30)
    }));

    // scheduling order will affect the index accessing poll1 and poll2 by poll3
    cahotic.schedule_after(&mut poll3, &mut poll1).unwrap(); // index 0
    cahotic.schedule_after(&mut poll3, &mut poll2).unwrap(); // index 1

    cahotic.schedule_exec(poll3);
    cahotic.schedule_exec(poll2);
    cahotic.schedule_exec(poll1);

    cahotic.join();
}
