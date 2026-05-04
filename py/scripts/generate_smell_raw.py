#!/usr/bin/env python3
"""
generate_smell_raw.py

Generates raw/code-smells/<kebab-name>.md for every SMELL entity defined in
meta/code_smells.json. Titles for RF-* and LAW-* IDs are resolved from
meta/relations.json.

Usage:
    python3 py/scripts/generate_smell_raw.py
"""

import json
import re
from pathlib import Path

# ---------------------------------------------------------------------------
# Paths (relative to repository root)
# ---------------------------------------------------------------------------
REPO_ROOT = Path(__file__).resolve().parent.parent.parent
META_DIR = REPO_ROOT / "meta"
RAW_SMELLS_DIR = REPO_ROOT / "raw" / "code-smells"

SMELLS_JSON = META_DIR / "code_smells.json"
RELATIONS_JSON = META_DIR / "relations.json"

# ---------------------------------------------------------------------------
# Category → human-readable label
# ---------------------------------------------------------------------------
CATEGORY_LABELS = {
    "bloater": "Bloater",
    "oo-abuser": "OO Abuser",
    "change-preventer": "Change Preventer",
    "dispensable": "Dispensable",
    "coupler": "Coupler",
    "inheritance": "Inheritance Abuser",
}

# Category → "Why It Matters" narrative template
CATEGORY_NARRATIVE = {
    "bloater": (
        "Bloater smells arise when a method, class, or parameter list grows "
        "to a size that makes it hard to read and maintain. As a **{category}**, "
        "this smell degrades comprehensibility and increases the cognitive load "
        "required to understand even small changes."
    ),
    "oo-abuser": (
        "OO Abuser smells indicate that object-oriented principles are not being "
        "applied correctly or are being actively circumvented. As an **{category}**, "
        "this smell undermines the polymorphism and encapsulation that make OO designs "
        "resilient to change."
    ),
    "change-preventer": (
        "Change Preventer smells make the codebase brittle: a single logical change "
        "forces edits in many unrelated places. As a **{category}**, this smell "
        "increases the risk and cost of every future modification, slowing down "
        "development over time."
    ),
    "dispensable": (
        "Dispensable smells point to code that is unnecessary and whose removal "
        "would make the codebase cleaner and easier to understand. As a **{category}**, "
        "this smell adds noise, cognitive overhead, and maintenance burden without "
        "providing corresponding value."
    ),
    "coupler": (
        "Coupler smells indicate excessive coupling between classes or modules, "
        "reducing cohesion and making it difficult to change one part of the system "
        "without affecting others. As a **{category}**, this smell works against the "
        "goal of designing components that can be understood, tested, and modified "
        "in isolation."
    ),
    "inheritance": (
        "Inheritance Abuser smells signal that the inheritance hierarchy is being "
        "misused or that subclasses do not honour the contract defined by their "
        "superclass. As an **{category}**, this smell can lead to fragile hierarchies "
        "and subtle runtime bugs when callers rely on substitutability that the "
        "subclass does not provide."
    ),
}

# ---------------------------------------------------------------------------
# LAW title → short description for "Why It Matters"
# ---------------------------------------------------------------------------
LAW_VIOLATION_CONTEXT = {
    "LAW-040": "the **DRY principle** (Don't Repeat Yourself), promoting unnecessary duplication and divergence",
    "LAW-041": "the **KISS principle** (Keep It Simple), adding unnecessary complexity",
    "LAW-042-S": "the **Single Responsibility Principle (SRP)**, giving the construct more than one reason to change",
    "LAW-042-O": "the **Open/Closed Principle (OCP)**, requiring modification of existing code to accommodate new behaviour",
    "LAW-042-L": "the **Liskov Substitution Principle (LSP)**, breaking substitutability between a type and its subtypes",
    "LAW-042-I": "the **Interface Segregation Principle (ISP)**, forcing clients to depend on interfaces they do not use",
    "LAW-042-D": "the **Dependency Inversion Principle (DIP)**, coupling high-level modules to low-level details",
    "LAW-042": "the **SOLID Principles**, undermining the foundations of good object-oriented design",
    "LAW-043": "the **Law of Demeter**, exposing implementation details across object boundaries",
    "LAW-044": "the **Principle of Least Astonishment**, surprising readers with unexpected behaviour",
}


def to_kebab(name: str) -> str:
    """Convert a SMELL name to a kebab-case filename stem."""
    name = name.lower()
    name = re.sub(r"[^a-z0-9\s-]", "", name)
    name = re.sub(r"\s+", "-", name.strip())
    return name


def load_json(path: Path) -> dict:
    with path.open(encoding="utf-8") as fh:
        return json.load(fh)


def build_rf_title_map(relations: dict) -> dict[str, str]:
    return {k: v["title"] for k, v in relations.items() if k.startswith("RF-")}


def build_law_title_map(relations: dict) -> dict[str, str]:
    return {k: v["title"] for k, v in relations.items() if k.startswith("LAW-")}


def render_symptoms(symptoms: list[str]) -> str:
    return "\n".join(f"- {s}" for s in symptoms)


def render_why_it_matters(smell: dict, law_map: dict) -> str:
    category = smell["category"]
    template = CATEGORY_NARRATIVE.get(
        category,
        "This smell negatively affects code quality and maintainability.",
    )
    paragraph1 = template.format(category=CATEGORY_LABELS.get(category, category))

    violates = smell.get("violates", [])
    if violates:
        violation_phrases = []
        for law_id in violates:
            ctx = LAW_VIOLATION_CONTEXT.get(
                law_id,
                f"**{law_map.get(law_id, law_id)}**",
            )
            violation_phrases.append(ctx)
        if len(violation_phrases) == 1:
            viol_sentence = f"Specifically, it violates {violation_phrases[0]}."
        else:
            joined = ", ".join(violation_phrases[:-1]) + f", and {violation_phrases[-1]}"
            viol_sentence = f"Specifically, it violates {joined}."
        paragraph2 = viol_sentence
    else:
        paragraph2 = ""

    return f"{paragraph1}\n\n{paragraph2}".strip()


def render_how_to_fix(smell: dict, rf_map: dict) -> str:
    solved_by = smell.get("solved_by", [])
    if not solved_by:
        return "No standard refactoring has been catalogued for this smell yet."
    lines = []
    for rf_id in solved_by:
        title = rf_map.get(rf_id, rf_id)
        lines.append(f"- **{title}** (`{rf_id}`)")
    return "\n".join(lines)


def render_connections(smell: dict, law_map: dict, rf_map: dict) -> str:
    parts = []

    violates = smell.get("violates", [])
    if violates:
        law_names = [f"{law_map.get(v, v)} (`{v}`)" for v in violates]
        parts.append("**Violates:** " + ", ".join(law_names))

    solved_by = smell.get("solved_by", [])
    if solved_by:
        rf_names = [f"{rf_map.get(r, r)} (`{r}`)" for r in solved_by]
        parts.append("**Resolved by refactoring:** " + ", ".join(rf_names))

    return "\n\n".join(parts) if parts else "_No additional connections recorded._"


def render_smell_md(smell: dict, rf_map: dict, law_map: dict) -> str:
    name = smell["name"]
    description = smell.get("description", "")
    symptoms = smell.get("symptoms", [])

    sections = [
        f"# {name}",
        "",
        "## Overview",
        description,
        "",
        "## Symptoms",
        render_symptoms(symptoms),
        "",
        "## Why It Matters",
        render_why_it_matters(smell, law_map),
        "",
        "## How to Fix",
        render_how_to_fix(smell, rf_map),
        "",
        "## Connections",
        render_connections(smell, law_map, rf_map),
        "",
        "---",
        "",
        "*Based on: Refactoring (Fowler, 1999)*",
    ]
    return "\n".join(sections)


def update_file_to_entity(generated: list[tuple[str, str, str]]) -> None:
    """Register generated smell MD files in meta/file_to_entity.json."""
    fte_path = META_DIR / "file_to_entity.json"
    mapping: dict = load_json(fte_path)

    added = 0
    for smell_id, _name, filename in generated:
        key = f"code-smells/{filename}"
        if key not in mapping:
            mapping[key] = smell_id
            added += 1

    # Preserve original key order, append new entries at the end
    fte_path.write_text(
        json.dumps(mapping, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    print(f"  [file_to_entity] {added} new entries added ({fte_path.relative_to(REPO_ROOT)})")


def main() -> None:
    smells_data = load_json(SMELLS_JSON)
    relations_data = load_json(RELATIONS_JSON)

    rf_map = build_rf_title_map(relations_data)
    law_map = build_law_title_map(relations_data)

    RAW_SMELLS_DIR.mkdir(parents=True, exist_ok=True)

    smells: dict = smells_data.get("smells", {})
    generated: list[tuple[str, str, str]] = []

    for smell_id, smell in sorted(smells.items()):
        filename = to_kebab(smell["name"]) + ".md"
        out_path = RAW_SMELLS_DIR / filename
        content = render_smell_md(smell, rf_map, law_map)
        out_path.write_text(content, encoding="utf-8")
        generated.append((smell_id, smell["name"], filename))
        print(f"  [+] {smell_id:10s}  {filename}")

    print(f"\nGenerated {len(generated)} files in {RAW_SMELLS_DIR.relative_to(REPO_ROOT)}/")
    update_file_to_entity(generated)


if __name__ == "__main__":
    main()
