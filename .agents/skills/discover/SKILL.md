---
name: discover
description: Create the next numbered Japanese discovery stub from a short title when the user explicitly invokes this skill.
---

# Capture a Discovery

Follow `docs/README.md`. Read the relevant documents under `docs/design/` to
understand the current design. When the title touches an existing design choice
or its rationale, also read the relevant documents under `docs/decisions/`,
including decisions linked from the design documents. Do not read unrelated
documents merely to make the stub comprehensive.

Treat the short title in the user's request as the input. If none is given, ask
the user for one before creating anything.

## Create the stub

Take the highest `NNNN` prefix among the existing `docs/discussions/` entries, add
one, and zero-pad it to four digits. Create the directory with that number alone —
`docs/discussions/<NNNN>/`, not a `NNNN-title` name. `$discuss` renames it to
`NNNN-short-description` when it later reads the content.

Write `docs/discussions/<NNNN>/discovery.md` in Japanese, with no frontmatter:

- The title as given, verbatim, on the first line.
- Then one to three loose lines guessing what this might be about, inferred from
  the title and the relevant design and decision documents. Getting it wrong is
  fine — this is a placeholder for the user to rewrite, so mark it plainly as a
  guess (e.g. lead with `（推測）`).

Keep it tiny. Apart from the documentation above, do not read the codebase. Do
not add frontmatter, do not add tags, and do not start the discussion — that is
`$discuss`'s job.

Then tell me the path you created.
