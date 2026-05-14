# CAP Theorem

## Statement
The CAP Theorem establishes that a distributed data store can provide at most two of three guarantees simultaneously: Consistency, meaning every read returns the most recent write; Availability, meaning every request receives a non-error response; and Partition Tolerance, meaning the system continues to operate despite network failures that prevent some nodes from communicating. Because network partitions are a physical reality in any distributed system, the practical choice is between consistency and availability when a partition occurs.

## Origin
Eric Brewer, a professor of computer science at UC Berkeley and co-founder of Inktomi, presented the conjecture in his 2000 keynote "Towards Robust Distributed Systems" at the ACM Symposium on Principles of Distributed Computing. Seth Gilbert and Nancy Lynch of MIT published a formal proof in 2002, confirming that no distributed system can simultaneously guarantee all three properties during a network partition. The theorem has since become a cornerstone of distributed systems architecture, shaping the design of databases, messaging systems, and cloud infrastructure.

## Software Implications
When a network link between two data centers drops, the system faces an immediate decision: refuse some requests to preserve consistency, or accept writes at both sites and reconcile differences later to preserve availability. Banking ledgers, inventory systems for limited stock, and coordination services like ZooKeeper choose consistency — returning an error is preferable to allowing conflicting balances. Social media feeds, shopping cart counts, and DNS choose availability — stale data for a few seconds is acceptable, but an outage is not.

The theorem does not dictate a single permanent choice. Modern systems often operate in different modes depending on conditions. A database cluster might provide strong consistency during normal operation but degrade to eventual consistency during a partition, then repair inconsistencies once connectivity is restored. This nuanced interpretation, sometimes called the PACELC extension, recognizes that even when there is no partition, systems face a latency-versus-consistency tradeoff.

Designing against CAP requires architects to classify each operation by its tolerance for stale data. Read-heavy workloads with low staleness tolerance, such as user authentication lookups, benefit from strong consistency. Write-heavy workloads that can tolerate reconciliation, such as analytics event ingestion, benefit from high availability. Misclassifying a workload leads to either unnecessary downtime or silent data conflicts.

## Practical Guidance
- Classify each data type in your system by its consistency requirements; not all data warrants the same tradeoff.
- Design explicit fallback behavior for partition scenarios rather than discovering your CAP choice by accident during an outage.
- Use conflict-free replicated data types or version vectors where eventual consistency is chosen, so reconciliation is algorithmic rather than manual.
- Monitor partition detection and recovery metrics closely; the period immediately after a partition heals is when data inconsistencies surface.

## Common Misreadings
The most damaging misreading treats CAP as a binary switch: "we are a CP system" or "we are an AP system." Real distributed systems make per-operation, per-data-type tradeoffs and often shift along the spectrum as conditions change. Another error assumes that choosing availability means abandoning consistency entirely; most AP systems provide eventual consistency, which converges to a consistent state given enough time. A third misreading interprets the theorem as applying only to databases, when in fact any system that replicates state across network boundaries — caches, message queues, configuration stores — faces the same constraints.

## Interactions
The Fallacies of Distributed Computing describe the environmental conditions that make network partitions inevitable, which is the prerequisite that forces the CAP tradeoff to activate. Hyrum's Law means that whichever consistency or availability behavior a system exhibits during a partition, downstream consumers will come to depend on that behavior, making it difficult to change the tradeoff later. The Law of Leaky Abstractions surfaces when an application pretends a distributed store is local — the partition behavior leaks through as timeouts, stale reads, or write conflicts. Gall's Law advises building distributed data management incrementally, discovering the right consistency boundaries through production experience rather than speculating about them in advance.

---

*Based on: Brewer, "Towards robust distributed systems" (ACM PODC, 2000)*
