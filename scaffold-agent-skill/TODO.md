# TODO

## Docs site

- [ ] Set `site_url` in `zensical.toml` — currently the placeholder
      `https://www.example.com/`. Use the GitHub Pages URL
      (e.g. `https://<user>.github.io/scaffold-agent-skill/`) so canonical
      links and `sitemap.xml` are correct.
- [ ] Optionally uncomment `repo_url` / `repo_name` / `edit_uri` in
      `zensical.toml` for the header repo link and "edit this page".
- [ ] `.github/workflows/docs.yml` only deploys on `master` / `main`; the work
      is on `dev`. Merge to the deploy branch (or add `dev` to the trigger)
      before expecting Pages to update.
- [ ] Enable GitHub Pages for the repo (Settings → Pages → Source: GitHub
      Actions) so the `docs.yml` workflow can publish.

## Repo housekeeping

- [ ] Fill in `README.md` (currently empty) — at minimum a one-liner and a
      pointer to the docs.
- [ ] Update `CLAUDE.md`: it predates the docs setup and doesn't mention the
      `zensical` docs toolchain (`uv run zensical serve` / `build`) or that
      `pyproject.toml` / `uv.lock` exist only for that.
- [ ] Reconcile `.python-version` (pins `3.14`) with `pyproject.toml`
      (`requires-python = ">=3.11"`).
- [ ] Commit the docs work — everything from this session is still staged /
      unstaged.

## Follow-ups from `/init`

- [ ] Codex (`~/.codex/`) and Gemini CLI (`~/.gemini/`) configs are present on
      this machine. Run `/import` to review importable items.
