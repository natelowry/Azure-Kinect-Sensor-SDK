# queue

A class for queuing capture objects.

## Abstract

The queue class handles insertion and retrieval of captures on different threads. Threads retrieving captures from a queue can block on a timeout (or indefinitely) waiting for data to become available.

Queues have a finite depth and will automatically drop old captures when they are filled.

Queues can be enabled or disabled. Disabled queues will drop their data and unblock any waiting threads.

Queues add a reference to the capture objects they contain. The reference is transferred to the reader when the capture is retrieved.

## Interface

See [queue.h](../../include/internal/queue.h)
