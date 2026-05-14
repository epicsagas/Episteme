# Fallacies of Distributed Computing

## Statement
The Fallacies of Distributed Computing are eight mistaken assumptions that programmers routinely make when first designing systems that span multiple networked machines: that the network is reliable, that latency is zero, that bandwidth is infinite, that the network is secure, that topology never changes, that there is a single administrator, that transport cost is zero, and that the network is homogeneous. Each assumption, when left unchallenged, leads to fragility that surfaces only under real-world conditions — precisely when it is most costly to fix.

## Origin
L. Peter Deutsch, an engineer at Sun Microsystems, formulated the first seven fallacies in 1994 based on recurring failures he observed in distributed systems at Sun and elsewhere. James Gosling, the creator of Java, later noted that an eighth fallacy — network homogeneity — deserved inclusion. The list was never published as a formal paper; it circulated as internal Sun lore before gaining wide recognition through conference talks and community references. Its persistence stems from the observation that each generation of developers rediscovers these fallacies through production incidents.

## Software Implications
Assuming the network is reliable leads to systems that crash or hang when a switch fails, instead of retrying with backoff and circuit breakers. Assuming latency is zero produces synchronous call chains where a single slow downstream service stalls the entire request. Assuming bandwidth is infinite results in APIs that transfer full object graphs when a handful of fields would suffice. Each fallacy represents a gap between the developer's mental model — where remote calls behave like local function calls — and the physical reality of packets traversing cables, routers, and firewalls.

Security assumptions are especially dangerous. A system that trusts all traffic within its internal network is one misconfigured firewall rule away from a breach. Zero-trust networking, mutual TLS, and service mesh architectures exist specifically to counter the fallacy that the network perimeter provides adequate protection.

Topology change is a constant in cloud environments. Instances are created and destroyed by autoscalers, IP addresses rotate, and DNS entries update asynchronously. Systems that hardcode hostnames or assume stable peer sets break under normal operational churn. Service discovery mechanisms — DNS-based, consul-style, or via orchestration platforms — address this fallacy directly, but only when engineers recognize the need for them.

## Practical Guidance
- Design every inter-service call to handle failure: implement timeouts, retries with jitter, and circuit breakers as default practice, not as afterthoughts.
- Measure actual latency and bandwidth between your services in production and design payload sizes and call frequencies against those measurements.
- Encrypt traffic in transit and authenticate callers even inside your own network perimeter; treat the network as hostile by default.
- Use service discovery and avoid hardcoding network topology; expect endpoints to shift during normal operations.

## Common Misreadings
A frequent overcorrection treats the fallacies as reasons to avoid distributed systems entirely, building monoliths even when scale or geographic distribution demands distribution. The fallacies are not arguments against distribution; they are arguments for designing distribution honestly, with failure as a first-class concern. Another misreading assumes the list is exhaustive. In practice, engineers discover additional assumptions regularly — for example, that clocks are synchronized across nodes or that DNS resolution is instantaneous and permanent.

## Interactions
The CAP Theorem formalizes the unavoidable tradeoff that becomes visible once the first fallacy — network reliability — is acknowledged: if the network can partition, you must choose between consistency and availability. The Law of Leaky Abstractions describes the mechanism by which these fallacies cause harm: developers use remote-call abstractions that conceal network realities, and when those realities assert themselves, the abstraction leaks in disruptive ways. Hyrum's Law means that once a system has been deployed with behavior shaped by any of these fallacies — say, an undocumented assumption about packet ordering — changing that behavior breaks dependent systems. Gall's Law suggests the safest path forward: evolve distributed capabilities from a working simple system so that each fallacy is encountered and addressed incrementally rather than all at once.

---

*Based on: Deutsch & Gosling, "The Eight Fallacies of Distributed Computing" (Sun Microsystems, 1994)*
