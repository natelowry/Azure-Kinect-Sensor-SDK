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

EXTENDS Sequences

CONSTANTS
  Capture,
  QueueDepth,
  Readers, \* Set of all readers
  Writers  \* Set of all writers

ReaderStates == { "Blocked", "Idle" }

VARIABLES
  (** variables held by the queue **)
  enabled, \* Is the queue enabled
  q,       \* content of the queue
  (** variables held by readers **)
  reader_threads
  
 
TypeInvariant ==
  /\ enabled \in { 0, 1 }
  /\ q \in Seq(Capture)
  /\ reader_threads \in [Readers -> ReaderStates]
  
Init == /\ enabled = 0
        /\ q = << >>
        /\ reader_threads = [r \in Readers |-> "Idle"]


Enqueue(element) ==
  /\ enabled = 1
  /\ q' = Append(q, element) 
  /\ UNCHANGED enabled
  
Enable(state) ==
  /\ enabled' = state
  /\ q' = IF state = 0 THEN << >> ELSE q

Next ==
  \/ Enqueue(Capture)
  \/ Enable( 0 )
  
  
=============================================================================
\* Modification History
\* Last modified Fri Aug 09 18:45:27 PDT 2019 by brenta
\* Created Fri Aug 09 17:52:46 PDT 2019 by brenta
