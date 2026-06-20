---
name: docs-sync
description: Update Design Docs and add ADRs to reflect recent decisions/changes. Use after finishing a chunk of work (a feature, a scaffold, an architectural choice) when docs/design-docs and docs/adr should be brought up to date. Design docs are edited in place; each new architecturally-significant decision gets a new sequential ADR.
allowed-tools: Read, Edit, Write, Glob, Grep, Bash(git diff:*), Bash(git log:*)
---

# Sync ADRs and Design Docs

Bring the project docs up to date with the work just completed. Design Docs are
living documents (update in place); ADRs are an append-only decision log.

## Gather

1. Run `git diff` and `git log --oneline -10`, and use the current conversation,
   to see what was decided, introduced, and achieved.
2. List existing `docs/design-docs/*.md` and `docs/adr/*.md`. Note the highest
   ADR number.

## Design Docs (`docs/design-docs/`) — UPDATE in place

- Edit the relevant existing file(s) so they match current reality: `Status`,
  `Components`, `Next steps`, `Out of scope`, etc.
- Do not create near-duplicate docs. Add a new design doc only for a genuinely
  new subsystem.

## ADRs (`docs/adr/`) — ADD only

- One file per architecturally-significant decision. Skip routine changes
  (bug fixes, renames, doc tweaks).
- Filename `NNNN-kebab-title.md` using the next zero-padded sequential number.
- Body: `Status`, `Date` (today), `Context`, `Decision`, `Consequences`
  (+ `Related` links when useful). Keep it ~15-25 lines.
- ADRs are immutable. To change a past decision, add a new ADR and set the old
  one's `Status` to `Superseded by ADR NNNN` with reciprocal links. Changing
  that status line is the only edit allowed to an accepted ADR.

## Both

- Lightweight over detailed ("軽量 > 詳しい").
- Write in English, matching the repo's existing doc style.
- Do not commit unless explicitly asked.

## Report

End with a short list of the files added/updated and why.
