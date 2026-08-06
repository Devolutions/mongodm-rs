---
name: commit-and-pr-conventions
description: Follow this repository's commit and pull-request conventions. Use whenever creating a commit or opening, editing, or preparing to squash-merge a PR, including when the user only asks to commit, push, or open a PR.
---

# Commit and PR conventions

Treat the PR title and initial body as the exact commit message that will land through squash-and-merge.

## Commit message

- Follow Conventional Commits: `<type>[optional scope][!]: <description>`.
- Limit the subject to 50 characters and describe the resulting behavior.
- Separate the subject, body, and footers with blank lines.
- For standalone commits, wrap body lines at 72 characters. Explain why the
  change was needed, how it solves the problem, and any side effects.
- Put the ticket key or link in a footer, not the subject: `Issue: HB-8770`.
- Add `Co-authored-by: Name <email>` trailers for collaborators and include
  other applicable Conventional Commit footers.
- Use `fix` for bugs, `feat` for features, `build` for build/dependencies,
  `chore` for non-product tools/configuration, `ci` for automation, `docs`
  for documentation only, `style` for non-semantic edits, `refactor` for
  restructuring, `test` for tests, and `perf` for performance.
- Keep each non-squashed development commit coherent and conventional too.

Example:

```text
fix: preserve unchanged collated indexes

Normalize server-expanded collations before comparing index definitions.

Issue: HB-8770
```

## Pull request

- Set the PR title to the intended squash commit summary exactly.
- Set the initial PR body to the intended squash commit body and footers exactly. Exclude PR-only preambles, checklists, and implementation journals.
- Do not manually wrap PR body lines; GitHub handles display wrapping.
- Keep the initial post concise: explain the motivation and material behavior needed in permanent history.
- Put useful author detail—experiments, alternatives, extensive validation, and implementation chronology—in follow-up comments.
- Update the title and initial body as the implementation evolves so they always describe what will merge. Add further comments when preserving new historical context is useful.
- Before opening, updating, or merging, read `title + blank line + body` as one commit message and correct any mismatch.
