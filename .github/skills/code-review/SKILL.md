---
name: code-review
description: Review a diff or pull request using the repository's focused review skills. Use for code reviews and the review step requested by contribute or lead.
---

Apply `code-compressor` to the requested revision or diff.
Load the additional review skills for the surfaces it changes:

- Rust: `rust-style-reviewer`.
- Tests: `test-reviewer`.
- Dependencies or lockfiles: `dependency-reviewer`.
- Documentation or explanatory comments: `docs-compressor` and `prose-verifier`.
- Code governed by intent or a contract: `intent-drift-audit`.

Apply `skeptical-reviewer` last to the diff and retained findings.
