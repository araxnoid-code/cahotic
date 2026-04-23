use cahotic::{CahoticBuilder, DefaultOutput, DefaultSchedule, DefaultTask};

fn main() {
    let cahotic = CahoticBuilder::default::<usize>().build().unwrap();

    let mut tensor_c = cahotic.scheduling_create_initial(DefaultTask(|| {
        println!("done task {} + {} = {}", 10, 20, 10 + 20);
        DefaultOutput(10 + 20)
    }));

    //

    let mut tensor_d = cahotic.scheduling_create_schedule(DefaultSchedule(|dep| {
        let c = dep.get(0).unwrap();

        println!("done task {} + {} = {}", c.0, 20, c.0 + 20);
        DefaultOutput(c.0 + 20)
    }));
    cahotic
        .schedule_after(&mut tensor_d, &mut tensor_c)
        .unwrap();

    //

    let mut tensor_e = cahotic.scheduling_create_schedule(DefaultSchedule(|dep| {
        let c = dep.get(0).unwrap();
        let d = dep.get(1).unwrap();

        println!("done task {} + {} = {}", c.0, d.0, c.0 + d.0);
        DefaultOutput(c.0 + d.0)
    }));
    cahotic
        .schedule_after(&mut tensor_e, &mut tensor_c)
        .unwrap();
    cahotic
        .schedule_after(&mut tensor_e, &mut tensor_d)
        .unwrap();

    //

    cahotic.schedule_exec(tensor_e);
    cahotic.schedule_exec(tensor_d);
    cahotic.schedule_exec(tensor_c);

    cahotic.join();
}
