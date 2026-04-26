use cahotic::{DefaultOutput, DefaultSchedule, Job};

fn main() {
    let job = Job::create_job(DefaultSchedule(|_| DefaultOutput(10)));
}
