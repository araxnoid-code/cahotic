# Drop
Task data cleaning on `cahotic` is done simultaneously based on the packet it occupies. To enable dropping, packets have several important components
1. `drop_list`, the place where all task data will be dropped.
2. `head`, as a drop limit.
3. `done_counter`, to ensure all tasks are completed before being deleted. This is especially important for scheduling..

<img width="400" src="./../img/drop_bitmap_graf.png">

briefly it can be explained like this:
1. Before the packet is submitted, each incoming task will be converted into a `WaitingTask` and add a `done_counter` to the packet that contains it. The `WaitingTask` contains the required information, one of which is the index of the packet that the `WaitingTask` occupies.
2. When the `WaitingTask` is finished, the thread executing the `WaitingTask` will immediately reduce the `done_counter` from the packets that the task has taken in it.
3. When `done_counter` becomes 0, the thread will change the `drop-bitmap`, `drop-bitmap` is the same as other bitmaps with a special function, namely to indicate which packets are ready to be dropped.
4. The `drop-bitmap` will be checked periodically quickly because it is in bitmap form, when there is a packet that can be dropped via the `drop-bitmap`, the thread that can first will take it and start dropping the packet.
5. When the drop is complete, the `empty-bitmap` will be updated according to the dropped packet index which is ready to be reused to accommodate new tasks.
