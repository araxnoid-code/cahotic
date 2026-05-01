# Version/0.4.0
## opening
because scheduling in Cahotic version 0.3.1 has limitations, namely it can only create 64 schedules at one time and separate schedules in the scheduling cannot communicate with each other directly, so for uses that require large scheduling it will be very disrupted. Therefore cahotic version 0.4.0 removes `Schedule` and replaces it with `job`

## job
job, is a structure for scheduling tasks that are spawned, the concept is the same as the previous schedule structure but jobs have technical differences.

jobs do not use allo-schedule-bitmap and poll-schedule-bitmap which are limited to 64bit which means they can only store 64 schedules, jobs use the same concept as ring buffers in tasks, jobs will be called job ring buffers.

when a job is run, in contrast to the schedule which will be `considered` to be in the task ring buffer for cleaning purposes, the job will now not interfere with the task section and the `quota` mechanism for cleaning purposes can now be accessed by tasks and jobs without having to influence each other's ring buffers on tasks and jobs.

With this job there will be no problems regarding the size of scheduling at one time, but there are several drawbacks, namely blocking of the workers thread when the job ring buffer is full.

the size of the job ring buffer will be the same as the size of the task ring buffer. because the schedule has been deleted, things related to the schedule have now been removed.
