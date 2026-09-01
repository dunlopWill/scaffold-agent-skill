---
icon: lucide/rocket
---

# scaffold-agent-skill

A small Rust CLI that creates a new [agent skill](https://agentskills.io/home)
directory, already shaped the way the [Agent Skills
format](https://agentskills.io/specification) expects: a `SKILL.md` with
validated frontmatter, the standard subdirectories, a starter `evals/` suite,
and — optionally — a Python test harness and a scratch workspace.

The generated files carry the authoring conventions as comments and `TODO`
markers, so starting a skill is: scaffold, then fill in the blanks.

## Install

``` sh
cargo install --path .
```

Or run it straight from the repo without installing:

``` sh
cargo run -- <NAME> [OPTIONS]
```

!!! note "Toolchain"

    The crate is edition 2024 — build with a recent stable Rust
    (`cargo >= 1.85`).

## First skill

``` sh
scaffold-agent-skill pdf-summarizer \
  -d "Summarize PDF files. Use when the user asks to condense, digest, or extract key points from a PDF."
```

``` title="result"
pdf-summarizer/
├── SKILL.md                  # frontmatter + Workflow / When to use / Gotchas
├── assets/.gitkeep
├── references/.gitkeep
├── scripts/.gitkeep
├── tests/.gitkeep
└── evals/
    ├── evals.json            # starter eval suite
    └── eval_queries.json     # trigger-eval prompts, labeled should_trigger
pdf-summarizer-workspace/     # empty sibling scratch area
```

Open `pdf-summarizer/SKILL.md` and replace every `TODO`.

## Next

- **[Scaffolding a skill](scaffolding-a-skill.md)** — every flag, common
  recipes, and the `--python` project layout.
- Run `scaffold-agent-skill --help` for the canonical option list.
- **[Agent Skills specification](https://agentskills.io/specification)** — what
  goes in `SKILL.md` once it's scaffolded. See also the
  [quickstart](https://agentskills.io/skill-creation/quickstart).

## Docs

These docs are built with [Zensical](https://zensical.org). To work on them:

``` sh
uv run zensical serve      # live preview at http://localhost:8000
uv run zensical build      # write the static site to site/
```
