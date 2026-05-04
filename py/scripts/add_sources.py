#!/usr/bin/env python3
"""
Add source/license metadata to all entities in relations.json.

Run once:
  python3 scripts/add_sources.py
"""

import json
from pathlib import Path

RELATIONS_PATH = Path(__file__).parent.parent / "meta" / "relations.json"

# ---------------------------------------------------------------------------
# Source definitions
# ---------------------------------------------------------------------------

# Design Patterns — GoF book (reference / fair-use summary)
_GOF = {
    "title": "Design Patterns: Elements of Reusable Object-Oriented Software",
    "authors": ["Erich Gamma", "Richard Helm", "Ralph Johnson", "John Vlissides"],
    "year": 1994,
    "publisher": "Addison-Wesley",
    "license": "reference",
    "url": "https://www.oreilly.com/library/view/design-patterns-elements/0201633612/",
}

# Refactoring — Martin Fowler (reference / fair-use summary)
_FOWLER = {
    "title": "Refactoring: Improving the Design of Existing Code",
    "authors": ["Martin Fowler"],
    "year": 1999,
    "publisher": "Addison-Wesley",
    "license": "reference",
    "url": "https://refactoring.com/",
}

# Code Smells — also from Fowler's Refactoring book
_FOWLER_SMELLS = {
    "title": "Refactoring: Improving the Design of Existing Code",
    "authors": ["Martin Fowler"],
    "year": 1999,
    "publisher": "Addison-Wesley",
    "license": "reference",
    "url": "https://refactoring.guru/refactoring/smells",
}

# Engineering laws & principles — curated from multiple public sources
_PUBLIC_DOMAIN = {
    "license": "public-domain",
    "note": "Named law or principle in the public domain; curated summary.",
}

_BROOKS = {
    "title": "The Mythical Man-Month",
    "authors": ["Fred Brooks"],
    "year": 1975,
    "publisher": "Addison-Wesley",
    "license": "reference",
    "url": "https://en.wikipedia.org/wiki/Brooks%27s_law",
}

_CONWAY = {
    "title": "How Do Committees Invent?",
    "authors": ["Melvin Conway"],
    "year": 1968,
    "publisher": "Datamation",
    "license": "reference",
    "url": "https://www.melconway.com/Home/Conways_Law.html",
}

_KERNIGHAN = {
    "title": "The Elements of Programming Style",
    "authors": ["Brian Kernighan", "P.J. Plauger"],
    "year": 1974,
    "publisher": "McGraw-Hill",
    "license": "reference",
    "url": "https://en.wikipedia.org/wiki/Kernighan%27s_law",
}

_MARTIN_CLEAN = {
    "title": "Clean Code",
    "authors": ["Robert C. Martin"],
    "year": 2008,
    "publisher": "Prentice Hall",
    "license": "reference",
    "url": "https://www.oreilly.com/library/view/clean-code-a/9780136083238/",
}

_FOWLER_REFACTORING_BLOG = {
    "title": "TechnicalDebt",
    "authors": ["Martin Fowler"],
    "license": "reference",
    "url": "https://martinfowler.com/bliki/TechnicalDebt.html",
}

_LEHMAN = {
    "title": "Laws of Software Evolution Revisited",
    "authors": ["Meir M. Lehman"],
    "year": 1996,
    "license": "reference",
    "url": "https://en.wikipedia.org/wiki/Lehman%27s_laws_of_software_evolution",
}

_AMDAHL = {
    "title": "Validity of the Single Processor Approach to Achieving Large-Scale Computing Capabilities",
    "authors": ["Gene Amdahl"],
    "year": 1967,
    "publisher": "AFIPS Conference Proceedings",
    "license": "reference",
    "url": "https://en.wikipedia.org/wiki/Amdahl%27s_law",
}

_GUSTAFSON = {
    "title": "Reevaluating Amdahl's Law",
    "authors": ["John L. Gustafson"],
    "year": 1988,
    "publisher": "Communications of the ACM",
    "license": "reference",
    "url": "https://en.wikipedia.org/wiki/Gustafson%27s_law",
}

_METCALFE = {
    "title": "Metcalfe's Law",
    "authors": ["Robert Metcalfe"],
    "year": 1980,
    "license": "reference",
    "url": "https://en.wikipedia.org/wiki/Metcalfe%27s_law",
}

_HYRUM = {
    "title": "Hyrum's Law",
    "authors": ["Hyrum Wright"],
    "license": "reference",
    "url": "https://www.hyrumslaw.com/",
}

_GALL = {
    "title": "Systemantics: How Systems Work and Especially How They Fail",
    "authors": ["John Gall"],
    "year": 1975,
    "publisher": "Quadrangle",
    "license": "reference",
    "url": "https://en.wikipedia.org/wiki/John_Gall_(author)#Gall%27s_law",
}

_ZAWINSKI = {
    "title": "Zawinski's Law",
    "authors": ["Jamie Zawinski"],
    "license": "reference",
    "url": "https://en.wikipedia.org/wiki/Jamie_Zawinski#Zawinski's_law",
}

_TESLER = {
    "title": "Tesler's Law (Law of Conservation of Complexity)",
    "authors": ["Larry Tesler"],
    "license": "reference",
    "url": "https://en.wikipedia.org/wiki/Law_of_conservation_of_complexity",
}

_MARTIN_SOLID = {
    "title": "Agile Software Development: Principles, Patterns, and Practices",
    "authors": ["Robert C. Martin"],
    "year": 2002,
    "publisher": "Prentice Hall",
    "license": "reference",
    "url": "https://en.wikipedia.org/wiki/SOLID",
}

_DEMETER = {
    "title": "Law of Demeter",
    "authors": ["Karl Lieberherr", "Ian Holland"],
    "year": 1987,
    "publisher": "OOPSLA",
    "license": "reference",
    "url": "https://en.wikipedia.org/wiki/Law_of_Demeter",
}

_HUNT_THOMAS = {
    "title": "The Pragmatic Programmer",
    "authors": ["Andrew Hunt", "David Thomas"],
    "year": 1999,
    "publisher": "Addison-Wesley",
    "license": "reference",
    "url": "https://pragprog.com/titles/tpp20/the-pragmatic-programmer-20th-anniversary-edition/",
}

# ---------------------------------------------------------------------------
# Per-entity source overrides (entity_id → source dict)
# Entities not listed here get the type-level default.
# ---------------------------------------------------------------------------

_ENTITY_SOURCES: dict[str, dict] = {
    # --- Teams ---
    "LAW-001": _CONWAY,
    "LAW-002": _BROOKS,
    "LAW-003": _PUBLIC_DOMAIN,   # Dunbar's Number (Robin Dunbar, anthropology)
    "LAW-004": _PUBLIC_DOMAIN,   # Ringelmann Effect
    "LAW-005": _PUBLIC_DOMAIN,   # Price's Law
    "LAW-006": _PUBLIC_DOMAIN,   # Putt's Law
    "LAW-007": _PUBLIC_DOMAIN,   # Peter Principle
    "LAW-008": _PUBLIC_DOMAIN,   # Bus Factor
    "LAW-009": _PUBLIC_DOMAIN,   # Dilbert Principle
    # --- Planning ---
    "LAW-010": _PUBLIC_DOMAIN,   # Premature Optimization (Knuth)
    "LAW-011": _PUBLIC_DOMAIN,   # Parkinson's Law
    "LAW-012": _PUBLIC_DOMAIN,   # Ninety-Ninety Rule
    "LAW-013": _PUBLIC_DOMAIN,   # Hofstadter's Law
    "LAW-014": _PUBLIC_DOMAIN,   # Goodhart's Law
    "LAW-015": _PUBLIC_DOMAIN,   # Gilb's Law
    # --- Architecture ---
    "LAW-016": _HYRUM,
    "LAW-017": _GALL,
    "LAW-018": {                  # Leaky Abstractions — Joel Spolsky
        "title": "The Law of Leaky Abstractions",
        "authors": ["Joel Spolsky"],
        "year": 2002,
        "license": "reference",
        "url": "https://www.joelonsoftware.com/2002/11/11/the-law-of-leaky-abstractions/",
    },
    "LAW-019": _TESLER,
    "LAW-020": _PUBLIC_DOMAIN,   # CAP Theorem (Brewer)
    "LAW-021": _PUBLIC_DOMAIN,   # Second System Effect (Brooks)
    "LAW-022": _PUBLIC_DOMAIN,   # Fallacies of Distributed Computing
    "LAW-023": _PUBLIC_DOMAIN,   # Law of Unintended Consequences
    "LAW-024": _ZAWINSKI,
    # --- Quality ---
    "LAW-025": _MARTIN_CLEAN,    # Boy Scout Rule
    "LAW-026": _PUBLIC_DOMAIN,   # Murphy's Law
    "LAW-027": _PUBLIC_DOMAIN,   # Postel's Law (Robustness Principle)
    "LAW-028": _PUBLIC_DOMAIN,   # Broken Windows Theory
    "LAW-029": _FOWLER_REFACTORING_BLOG,  # Technical Debt
    "LAW-030": _PUBLIC_DOMAIN,   # Linus's Law
    "LAW-031": _KERNIGHAN,
    "LAW-032": _PUBLIC_DOMAIN,   # Testing Pyramid (Mike Cohn)
    "LAW-033": _PUBLIC_DOMAIN,   # Pesticide Paradox (Beizer)
    "LAW-034": _LEHMAN,
    "LAW-035": _PUBLIC_DOMAIN,   # Sturgeon's Law
    "LAW-036": _PUBLIC_DOMAIN,   # YAGNI (XP / Ron Jeffries)
    # --- Scalability ---
    "LAW-037": _AMDAHL,
    "LAW-038": _GUSTAFSON,
    "LAW-039": _METCALFE,
    # --- Design ---
    "LAW-040": _HUNT_THOMAS,     # DRY
    "LAW-041": _PUBLIC_DOMAIN,   # KISS
    "LAW-042": _MARTIN_SOLID,    # SOLID
    "LAW-043": _DEMETER,
    "LAW-044": _PUBLIC_DOMAIN,   # Principle of Least Astonishment
    # --- Decisions ---
    "LAW-045": _PUBLIC_DOMAIN,   # Dunning-Kruger Effect
    "LAW-046": _PUBLIC_DOMAIN,   # Hanlon's Razor
    "LAW-047": _PUBLIC_DOMAIN,   # Occam's Razor
    "LAW-048": _PUBLIC_DOMAIN,   # Sunk Cost Fallacy
    "LAW-049": _PUBLIC_DOMAIN,   # Map Is Not Territory
    "LAW-050": _PUBLIC_DOMAIN,   # Confirmation Bias
    "LAW-051": _PUBLIC_DOMAIN,   # Hype Cycle / Amara's Law
    "LAW-052": _PUBLIC_DOMAIN,   # Lindy Effect
    "LAW-053": _PUBLIC_DOMAIN,   # First Principles Thinking
    "LAW-054": _PUBLIC_DOMAIN,   # Inversion
    "LAW-055": _PUBLIC_DOMAIN,   # Pareto Principle
    "LAW-056": _PUBLIC_DOMAIN,   # Cunningham's Law
}

# Type-level defaults (used when no per-entity override exists)
_TYPE_DEFAULTS: dict[str, dict] = {
    "pattern":     _GOF,
    "refactoring": _FOWLER,
    "smell":       _FOWLER_SMELLS,
    "law":         _PUBLIC_DOMAIN,
}


def main() -> None:
    with open(RELATIONS_PATH, encoding="utf-8") as f:
        data = json.load(f)

    updated = 0
    for entity_id, entity in data.items():
        if not isinstance(entity, dict):
            continue
        if "source" in entity:
            continue  # already set

        entity_type = entity.get("type", "")
        source = _ENTITY_SOURCES.get(entity_id) or _TYPE_DEFAULTS.get(entity_type)
        if source:
            entity["source"] = source
            updated += 1

    with open(RELATIONS_PATH, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)

    print(f"Updated {updated} entities → {RELATIONS_PATH}")


if __name__ == "__main__":
    main()
