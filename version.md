# Version/0.1.0
This update is dominated by changes to the internal system of cahotic, especially in the scheduling and drop-packet execution mechanisms.

1. Updated the `Scheduling` mechanism.
- Added `schedule-list`. `schedule-list` is a 64-sized array that will store schedules that have not yet been executed.
- Added `allo-schedule-bitmap`. as a marker and signal for `packet-core` to store the schedule that has been created.
- Addition of `poll-schedule-bitmap`. as a marker and signal for threads in the thread pool to execute schedules that are ready to be executed.
2. Updated the `drop` mechanism
- addition of `drop-bitmap`, as a place to provide signals and positions to threads in the `thread-pool` of packets that are ready to be dropped.
3. Idle-handling
- When a thread has no tasks for some time, it will take a short `rest` to give the CPU a breather.
4. removes the old concept of requiring a local queue for each thread for drop-packet handling and scheduler.
5. added English and Indonesian language guides.
