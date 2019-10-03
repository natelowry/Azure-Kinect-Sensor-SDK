------------------------------- MODULE queue -------------------------------

(**************************************************************************)
(* queue                                                                  *)
(*                                                                        *)
(* queue may have a set of reader threads and writer threads              *)
(* simultaneously attempt to enqueue and dequeue captures.                *)
(*                                                                        *)
(* queue must support threads blocking infinitely without the system      *)
(* deadlocking.                                                           *)
(*                                                                        *)
(* disabling or destroying the queue will release all threads             *)
(*                                                                        *)
(*                                                                        *)
(**************************************************************************)

EXTENDS Sequences, Integers

CONSTANTS
  Capture,
  QueueDepth,
  Readers \* Set of all readers

\* States that the readers may be in
\*    RetrievingCapture = Waiting on a new capture
\*    Other = Anything other than retrieving a capture (processing, unrelated tasks, etc)
\*    Exited = Thread has completed
ReaderStates == { "RetrievingCapture", "Other", "Exited" }


\* Reader thread pattern
\* while (running) {
\*   if (failed(pop())
\*      break;
\*   // do other work
\* }

\* shutdown pattern
\* enable(0)
\* wait for all threads
\* destroy queue



\* Reader threads will loop betwen trying to read a capture, and doing other work.
\* If no sample is availabe, the thread will block until one is ready
\* If the system is being shutdown the thread will transition to exit, if there
\* is an error it will also transition to exit

VARIABLES
  (** variables held by the queue **)
  enabled,   \* Is the queue enabled
  q,         \* content of the queue
  destroyed, \* Is the queue destroyed
  (** variables held by readers **)
  reader_threads,
  (** globals managed by shutdown thread **)
  running,   \* Signal to terminate reader threads
  shutdown_state
 
TypeInvariant ==
  /\ enabled \in { 0, 1 }
  /\ q \in Seq(Capture)
  /\ reader_threads \in [Readers -> ReaderStates]
  /\ running \in { 0, 1 }
  /\ destroyed \in { 0, 1 }
  /\ shutdown_state \in { "Running", "WaitingForThreads", "Done" }
 
InvalidAccessInvariant == TRUE
\*  /\ ((\A r \in Readers : reader_threads[r] /= "RetrievingCapture") \/ destroyed = 0)
  
Init ==
  /\ enabled = 1
  /\ q = << >>
  /\ reader_threads = [r \in Readers |-> "Other"]
  /\ running = 1
  /\ destroyed = 0
  /\ shutdown_state = "Running"

Enqueue(element) ==
  /\ enabled = 1
  /\ q' = IF Len(q) < QueueDepth THEN Append(q, element) ELSE Append(SubSeq(q, 2, Len(q)), element)
  /\ UNCHANGED << enabled, reader_threads, running, destroyed, shutdown_state >>
  
Enable(state) ==
  /\ enabled' = state
  /\ q' = IF state = 0 THEN << >> ELSE q
  /\ UNCHANGED << reader_threads, running, destroyed >>

\* Intentional bug: Don't check the running state, and uncondionally try getting a capture
StartRead(i) ==
  /\ reader_threads[i] = "Other"
  /\ reader_threads' = [reader_threads EXCEPT ![i] = "RetrievingCapture" ] 
  /\ UNCHANGED << q, enabled, running, destroyed, shutdown_state >>

EndReadEnabled(i) ==
  /\ reader_threads[i] = "RetrievingCapture"
  /\ enabled = 1
  /\ Len(q) > 0
  /\ reader_threads' = [reader_threads EXCEPT ![i] = "Other" ]
  /\ q' = SubSeq(q, 2, Len(q))
  /\ UNCHANGED << enabled, running, destroyed, shutdown_state >>

EndReadDisabled(i) ==
  /\ reader_threads[i] = "RetrievingCapture"
  /\ enabled = 0
  /\ reader_threads' = [reader_threads EXCEPT ![i] = "Exited" ]
  /\ UNCHANGED << q, enabled, running, destroyed, shutdown_state >>

DisableQueue ==
  /\ shutdown_state = "Running"
  /\ shutdown_state' = "WaitingForThreads"
  /\ enabled' = 0
  /\ q' = << >>
  /\ running' = 0
  /\ UNCHANGED << reader_threads, destroyed >>

FinishWaitingForThreads ==
  /\ shutdown_state = "WaitingForThreads"
  /\ \A r \in Readers : reader_threads[r] = "Exited"
  /\ destroyed' = 1
  /\ shutdown_state' = "Done"
  /\ UNCHANGED << q, enabled, running, reader_threads >>
  
Done ==
  /\ shutdown_state = "Done"
  /\ shutdown_state' = shutdown_state
  /\ UNCHANGED << q, enabled, running, reader_threads, destroyed >>


WriterProcess ==
    \E c \in Capture : Enqueue( c )

ReaderProcess(i) ==
  StartRead(i) \/ EndReadEnabled(i) \/ EndReadDisabled(i)
    
ShutdownProcess == DisableQueue \/ FinishWaitingForThreads \/ Done
  
Next ==
  \/ WriterProcess
  \/ \E r \in Readers : ReaderProcess( r )
  \/ ShutdownProcess
  
  
=============================================================================
\* Modification History
\* Last modified Sun Aug 11 18:28:53 PDT 2019 by brenta
\* Created Fri Aug 09 17:52:46 PDT 2019 by brenta
