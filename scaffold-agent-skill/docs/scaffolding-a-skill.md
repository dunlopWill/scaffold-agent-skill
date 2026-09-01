---
icon: lucide/folder-plus
---

# Scaffolding an agent skill

`scaffold-agent-skill` is a small Rust CLI that lays down a new
[agent skill](https://agentskills.io/home) directory: a `SKILL.md` with
validated frontmatter, the standard subdirectories, an `evals/` suite, and
— optionally — a Python test harness and a sibling scratch workspace.

The generated files encode the skill-authoring conventions, so the fastest way
to start a new skill correctly is to scaffold it and then fill in the `TODO`
markers.

!!! info "The Agent Skills format"

    A skill is a folder with a `SKILL.md` at its root. This CLI produces the
    layout and frontmatter that format expects — see the
    [Agent Skills specification](https://agentskills.io/specification) for the
    full rules on `name`, `description`, and the optional frontmatter fields,
    and the [quickstart](https://agentskills.io/skill-creation/quickstart) for
    a walkthrough.

## Install

=== "Run from the repo"

    ``` sh
    cargo run -- <NAME> [OPTIONS]
    ```

=== "Install the binary"

    ``` sh
    cargo install --path .
    scaffold-agent-skill <NAME> [OPTIONS]
    ```

=== "Build a release binary"

    ``` sh
    cargo build --release
    ./target/release/scaffold-agent-skill <NAME> [OPTIONS]
    ```

!!! note "Toolchain"

    The crate is edition 2024; build it with a recent stable Rust
    (`cargo >= 1.85`).

## Quick start

``` sh title="Create a skill in the current directory"
scaffold-agent-skill pdf-summarizer \
  -d "Summarize PDF files. Use when the user asks to condense, digest, or extract key points from a PDF."
```

This writes:

``` title="pdf-summarizer/"
pdf-summarizer/
├── SKILL.md                  # frontmatter + Workflow / When to use / Gotchas
├── assets/.gitkeep
├── references/.gitkeep
├── scripts/.gitkeep
├── tests/.gitkeep
└── evals/
    ├── evals.json            # starter eval suite (activates / skips)
    └── eval_queries.json     # trigger-eval prompts, labeled should_trigger
pdf-summarizer-workspace/     # empty sibling scratch area
```

Then open `pdf-summarizer/SKILL.md` and replace every `TODO`.

## Arguments and options

| Argument / option | Default | Purpose |
| --- | --- | --- |
| `<NAME>` | — | Skill name and directory name. 1–64 chars, lowercase `a–z`, `0–9`, and `-`; no leading/trailing or consecutive hyphens. Validated before anything is written. |
| `-d`, `--description <TEXT>` | a `TODO` placeholder | One-line `description` for the `SKILL.md` frontmatter. Written as a quoted YAML scalar, so colons and `#` are safe. Non-empty, max 1024 chars. |
| `-p`, `--path <DIR>` | `.` | Parent directory the skill directory is created in. |
| `--dirs <A,B,C>` | `references,scripts,tests,assets` | Comma-separated subdirectories to create inside the skill. Each gets a `.gitkeep`. |
| `--no-evals` | off | Skip `evals/` and its JSON files. |
| `--minimal` | off | Write only `SKILL.md` — no subdirectories, no `evals/`. Overrides `--dirs`. |
| `--python` | off | Also scaffold a Python project (see below). Conflicts with `--minimal`. |
| `--no-workspace` | off | Do not create the sibling `<name>-workspace/` directory. |
| `-f`, `--force` | off | Overwrite `SKILL.md` / `evals/*.json` instead of erroring when they already exist. |

Run `scaffold-agent-skill --help` for the canonical list.

## Common recipes

``` sh title="Skill in a dedicated skills directory"
scaffold-agent-skill release-notes -p ~/.claude/skills \
  -d "Draft release notes from merged PRs. Use when asked to summarize what shipped."
```

``` sh title="Just a SKILL.md, nothing else"
scaffold-agent-skill quick-lookup --minimal --no-workspace -d "..."
```

``` sh title="Custom subdirectories"
scaffold-agent-skill data-loader --dirs references,fixtures -d "..."
```

## Python skills

Add `--python` when the skill ships executable scripts:

``` sh
scaffold-agent-skill csv-cleaner --python -d "Clean and normalize CSV files."
```

On top of the default tree you also get:

``` title="extra files under --python"
csv-cleaner/
├── pyproject.toml            # pytest + a dev dependency group
├── .gitignore                # __pycache__, .venv, caches
├── scripts/example.py        # self-contained PEP 723 starter script (executable)
└── tests/test_skill.py       # green pytest suite exercising example.py
```

`scripts/` and `tests/` get these real starter files **instead of** a
`.gitkeep`. The script follows the conventions an agent-invoked tool should
follow: a useful `--help`, JSON on stdout, diagnostics on stderr, `--dry-run`,
`--output`, and distinct exit codes.

``` sh title="Run a Python skill's tooling"
cd csv-cleaner
uv run scripts/example.py --help
uv run pytest
```

## Behavior notes

- **Validation happens before any file is written.** A bad name or empty
  description leaves the filesystem untouched.
- **Collisions are detected up front.** Without `--force`, if `SKILL.md` or an
  `evals/*.json` file already exists the command aborts before writing anything
  — it does not leave a half-written skill behind.
- **The directory is always named after `<NAME>`**, which keeps the frontmatter
  `name` in sync with the directory name as the skill format requires.
- The sibling `<name>-workspace/` is created empty on purpose: a scratch area
  for iterating on the skill without cluttering the skill directory itself.
