--------------------------- MODULE queue_pluscal ---------------------------

EXTENDS TLC, Sequences, Integers

CONSTANTS Data

(* --algorithm queue_pluscal

variables q = << >>;


macro push(e) begin
    q := IF Len(q) < 2 THEN Append(q, e) ELSE Append(SubSeq(q, 2, Len(q)), e);
end macro

macro pop() begin
    await Len(q) > 0;
    q := SubSeq(q, 2, Len(q));
end macro

process read_queue \in 1..2
begin
 RA:
  while TRUE do
    RB:
     pop();
  end while
end process;

process write_queue = 3
begin
 WA:
  while TRUE do
    WB:
     push(0);
  end while
end process;

end algorithm; *)


\* BEGIN TRANSLATION
VARIABLES q, pc

vars == << q, pc >>

ProcSet == (1..2) \cup {3}

Init == (* Global variables *)
        /\ q = << >>
        /\ pc = [self \in ProcSet |-> CASE self \in 1..2 -> "RA"
                                        [] self = 3 -> "WA"]

RA(self) == /\ pc[self] = "RA"
            /\ pc' = [pc EXCEPT ![self] = "RB"]
            /\ q' = q

RB(self) == /\ pc[self] = "RB"
            /\ Len(q) > 0
            /\ q' = SubSeq(q, 2, Len(q))
            /\ pc' = [pc EXCEPT ![self] = "RA"]

read_queue(self) == RA(self) \/ RB(self)

WA == /\ pc[3] = "WA"
      /\ pc' = [pc EXCEPT ![3] = "WB"]
      /\ q' = q

WB == /\ pc[3] = "WB"
      /\ q' = (IF Len(q) < 2 THEN Append(q, 0) ELSE Append(SubSeq(q, 2, Len(q)), 0))
      /\ pc' = [pc EXCEPT ![3] = "WA"]

write_queue == WA \/ WB

Next == write_queue
           \/ (\E self \in 1..2: read_queue(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION

=============================================================================
\* Modification History
\* Last modified Sat Aug 10 09:29:06 PDT 2019 by brenta
\* Created Sat Aug 10 08:27:09 PDT 2019 by brenta
