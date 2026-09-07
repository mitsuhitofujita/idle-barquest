---
date: 2026-08-29
status: merged
discussions: []
decisions: []
pr: null
---

# Documentation model migration

## Plan

- Classify each existing document by its role and lifetime.
- Move accepted ADRs to `docs/decisions/` without changing their decisions.
- Move current implementation references to `docs/design/`.
- Preserve unimplemented game and progression drafts as discussion proposals.
- Update repository references and verify that no legacy paths remain.

## Outcome

- Moved ten ADRs to `docs/decisions/` and updated the two historical links that
  pointed at relocated design documents.
- Consolidated the architecture overview, TUI layout, and TUI test policy under
  `docs/design/`, then added their related-decision links.
- Reclassified the game design and early progression drafts as Japanese
  proposals under `docs/discussions/`.
- Updated Rust documentation comments to point at the new design paths.
- Removed the superseded `adr/`, `design-docs/`, and `plans/` directories.

## Difficulties and deviations

The migrated proposals predate the four-layer model. Their original source did
not contain a human-authored discovery or meeting minutes. Creating those files
retroactively would invent history, so each migrated discussion intentionally
contains only the preserved proposal.

The existing accepted ADRs are otherwise immutable. Two references inside them
had to follow relocated design files so that deleting the legacy paths would not
leave broken links; their architectural content was not changed.
