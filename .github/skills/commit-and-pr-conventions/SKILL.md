---
name: commit-and-pr-conventions
description: Follow this repository's commit and pull-request conventions. Use whenever creating a commit or opening, editing, or preparing to squash-merge a PR, including when the user only asks to commit, push, or open a PR.
---

# Commit and PR conventions

Treat the PR title and initial body as the exact commit message that will land through squash-and-merge.

## Commit message

- Follow Conventional Commits: `<type>[optional scope][!]: <description>`.
- Keep the summary concise and describe the resulting behavior.
- Put the ticket key in a footer, not the summary: `Issue: HB-8770`.
- Include a body when the motivation or behavior is not obvious.
- Keep each non-squashed development commit coherent and conventional too.

Example:

```text
fix: stop rebuilding collated indexes on every sync_indexes call

Normalize server-expanded collations before comparing index definitions.

Issue: HB-8770
```

## Pull request

- Set the PR title to the intended squash commit summary exactly.
- Set the initial PR body to the intended squash commit body and footers exactly. Exclude PR-only preambles, checklists, and implementation journals.
- Keep the initial post concise: explain the motivation and material behavior needed in permanent history.
- Put useful author detail—experiments, alternatives, extensive validation, and implementation chronology—in follow-up comments.
- Update the title and initial body as the implementation evolves so they always describe what will merge. Add further comments when preserving new historical context is useful.
- Before opening, updating, or merging, read `title + blank line + body` as one commit message and correct any mismatch.
