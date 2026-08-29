---
name: decide
description: Record a completed worklog as an accepted decision and update the current design documentation when the user explicitly asks to finalize the work.
---

# Finalize a Worklog

Follow `docs/README.md`.

Treat the worklog path in the user's request as the input. Accept either a
directory under `docs/worklogs/` or any file in that directory. Resolve a file
input to its containing worklog directory.

## Inspect the Sources

Read the worklog's `plan.md`. Resolve every discussion identifier in its
frontmatter. For each related discussion, read `proposal.md` and `minutes.md`
when they exist. Some migrated discussions intentionally lack one or both
files; do not invent missing history.

This repository currently commits directly to `main` without pull requests or
merges. Explicit invocation of this skill is the acceptance event, so do not
wait for a merge.

## Write the Decision

Create `docs/decisions/NNNN-short-title.md` in English. Use the next four-digit
number in `docs/decisions/` and an English kebab-case title.

Keep the document concise and use this structure:

```markdown
# ADR NNNN: Title

- Status: Accepted
- Date: YYYY-MM-DD
- Discussions: [discussion](../discussions/NNNN-short-title/proposal.md)
- Worklog: [worklog](../worklogs/NNNN-short-title/plan.md)

## Decision

...

## Rationale

...

## Related

...
```

Omit the Discussions line when the worklog has no related discussion. Include
all related discussions when there are several. Summarize only what was
accepted and why. Do not copy the proposal, repeat the discussion, or turn the
worklog into a narrative of the implementation.

If the decision supersedes an earlier decision, create the new decision first.
Then change the earlier decision's status to
`Superseded by [ADR NNNN](NNNN-short-title.md)` and add links in both directions.
Do not otherwise rewrite the earlier decision.

## Update the Current Design

If the accepted work changes the current system, update the relevant documents
under `docs/design/` in English. Replace obsolete descriptions because design
documents describe only the current state.

Add the new decision to each changed document's existing `Related decisions`
section, creating that section at the end only when it does not exist. Do not
duplicate links. If the current state did not change, leave `docs/design/`
untouched.

## Synchronize Status and Links

Use bare document identifiers in frontmatter relationship arrays, not Markdown
URLs.

- Add the new decision identifier to the worklog's `decisions` array.
- Set the worklog status to `merged`.
- For every related discussion that has a `discovery.md`, add the decision
  identifier to its `decisions` array and set its status to `decided`.
- If the worklog has no related discussion, do not create or update a
  discovery.
- Never change the body of a `discovery.md`.

## Report Remaining Difficulties

Point out any structural difficulty recorded in the worklog that does not yet
have a discovery. Finalizing the work does not resolve newly exposed design or
assumption problems.
