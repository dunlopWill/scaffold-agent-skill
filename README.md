# scaffold-agent-skill

A single-command CLI that lays down a new [agent skill](https://agentskills.io/specification)
directory: a `SKILL.md` with validated frontmatter, the standard
subdirectories, an `evals/` suite, and — optionally — a Python test harness and
a sibling scratch workspace.

The generated files encode the skill-authoring conventions, so the fastest way
to start a new skill correctly is to scaffold it and fill in the `TODO` markers.

## Install

```sh
cargo install scaffold-agent-skill
```

Or build from a checkout (edition 2024, needs `cargo >= 1.85`):

```sh
cargo build --release
./target/release/scaffold-agent-skill <NAME> [OPTIONS]
```

## Quick start

```sh
scaffold-agent-skill pdf-summarizer \
  -d "Summarize PDF files. Use when the user asks to condense, digest, or extract key points from a PDF."
```

writes:

```text
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

## Options

| Argument / option | Default | Purpose |
| --- | --- | --- |
| `<NAME>` | — | Skill name and directory name. 1–64 chars, lowercase `a–z`, `0–9`, `-`; no leading/trailing or consecutive hyphens. Validated before anything is written. |
| `-d`, `--description <TEXT>` | a `TODO` placeholder | One-line `description` for the `SKILL.md` frontmatter. Written as a quoted YAML scalar, so `:` and `#` are safe. Non-empty, max 1024 chars. |
| `-p`, `--path <DIR>` | `.` | Parent directory the skill directory is created in. |
| `--dirs <A,B,C>` | `references,scripts,tests,assets` | Comma-separated subdirectories to create inside the skill. Each gets a `.gitkeep`. |
| `--no-evals` | off | Skip `evals/` and its JSON files. |
| `--minimal` | off | Write only `SKILL.md` — no subdirectories, no `evals/`. Overrides `--dirs`. |
| `--python` | off | Also scaffold a Python project (`pyproject.toml`, `.gitignore`, `scripts/example.py`, `tests/test_skill.py`). Conflicts with `--minimal`. |
| `--no-workspace` | off | Do not create the sibling `<name>-workspace/` directory. |
| `-f`, `--force` | off | Overwrite `SKILL.md` / `evals/*.json` instead of erroring when they already exist. |

Run `scaffold-agent-skill --help` for the canonical list.

## Behavior notes

- **Validation happens before any file is written.** A bad name or empty
  description leaves the filesystem untouched.
- **Collisions are detected up front.** Without `--force`, if `SKILL.md` or an
  `evals/*.json` file already exists the command aborts before writing anything
  — it never leaves a half-written skill behind.
- **The directory is always named after `<NAME>`**, keeping the frontmatter
  `name` in sync with the directory name as the skill format requires.
- The sibling `<name>-workspace/` is created empty on purpose: a scratch area
  for iterating on the skill without cluttering the skill directory.

## Documentation

Full CLI reference: <https://github.com/dunlopWill/scaffold-agent-skill>

## License

MIT
