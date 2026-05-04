# Kernighan's Law

## Statement
Kernighan's Law holds that debugging code is twice as difficult as writing it in the first place. Therefore, if you write code that is as clever as possible, you will be unable to debug it when it breaks.

## Origin
Brian W. Kernighan, co-author of "The Elements of Programming Style" (1974) and "The C Programming Language" (1978), articulated this observation during decades of writing and teaching production-grade software. The insight appears in various forms across his writing and lectures, consistently arguing that the effort to understand code during debugging always exceeds the effort to write it. Kernighan's broader body of work champions clarity and simplicity as engineering virtues rather than aesthetic preferences.

## Software Implications
The law has direct consequences for code review culture. When a reviewer encounters a dense, clever one-liner, Kernighan's Law predicts that the author will struggle to diagnose failures in that same line weeks later. Teams that adopt readability as a first-class code quality metric — through style guides, review checklists, and explicit complexity budgets — are indirectly applying this law.

The principle also shapes language and tool design. Languages that prioritize readability over expressiveness (Go's deliberate simplicity, Python's "there should be one obvious way to do it") embody Kernighan's insight at the language level. Debuggers, structured logging, and observability tools exist precisely because the gap between writing and understanding code is wide enough to warrant dedicated tooling.

## Practical Guidance
- Write code as if the person debugging it at 3 AM is you, six months from now
- Favor straightforward implementations over clever ones, even if the clever version is shorter
- If a code block requires a comment explaining its cleverness, consider rewriting it to be obvious instead
- Use meaningful names and consistent patterns so that debugging becomes a search for logic errors, not a decryption exercise
- During code review, flag code that makes you pause to understand it — that pause is Kernighan's Law in action

## Common Misreadings
Some interpret the law as an argument against optimization or advanced algorithms entirely. Kernighan's point is not that you should avoid complexity when the problem demands it, but that the complexity should be concentrated in the right places and isolated behind clear interfaces. A high-performance sort algorithm is justified; a nested ternary expression in business logic is not. The law targets gratuitous cleverness, not necessary sophistication.

Another misunderstanding is treating the "twice as hard" ratio as literal. The actual difficulty multiplier depends on the domain, the time elapsed, and the number of people who have touched the code. The core insight — that reading and debugging code is systematically harder than writing it — holds regardless of the exact factor.

## Interactions
Kernighan's Law reinforces KISS by providing a concrete mechanism: clever code fails the debuggability test. It complements the Boy Scout Rule because leaving code cleaner than you found it directly reduces the debugging burden on the next person. The law interacts with Sturgeon's Law as a reminder that the "ninety percent" of mediocre code is often the result of developers prioritizing cleverness over clarity. YAGNI provides a natural ally: features you don't write don't need debugging.

---

*Based on: Kernighan & Plauger, "The Elements of Programming Style" (1974)*
