---
name: work
description: Create an English worklog and implement an agreed proposal selected by discussion number or path, or a direct task, when the user explicitly invokes this skill.
---

# Implement and Record Work

Follow `docs/README.md`.

Treat the argument in the user's request as the input.

## Inspect the Starting Point

Resolve the input in this order:

1. If it is a bare sequence number such as `10` or `0010`, zero-pad it to four
   digits. Find immediate subdirectories of `docs/discussions/` whose names are
   exactly that number or start with that number followed by `-`. Continue only
   when exactly one directory matches and it contains `proposal.md`; otherwise,
   stop and ask the user how to proceed.
2. If it is a repository-relative path to a directory or file under
   `docs/discussions/`, resolve it to the containing discussion directory and
   read that directory's `proposal.md`.
3. Otherwise, treat it as a description of a direct task. A small direct task
   does not require a proposal or a new discovery; do not ask the user to create
   one merely to satisfy this workflow.

Read the relevant documents under `docs/design/` before implementation. When a
migrated discussion lacks `discovery.md`, preserve that historical gap rather
than inventing one.

## Create the Worklog

Create `docs/worklogs/NNNN-short-description/plan.md`. Use the next four-digit
number in `docs/worklogs/` and an English kebab-case description. Numbering is
independent within `docs/worklogs/`; it does not continue the sequence from
discussions or decisions.

Use today's date and this frontmatter:

```yaml
---
date: YYYY-MM-DD
status: in-progress
discussions: []
decisions: []
pr: null
---
```

Use bare document identifiers in relationship arrays, not Markdown URLs. If
the work starts from one or more discussions, list every discussion identifier
in `discussions`. For every related discussion that has a `discovery.md`, add
the worklog identifier to its `worklogs` array. These links must be
bidirectional. If there is no discussion, leave `discussions` empty and do not
create a discovery.

## Implement and Record Progress

Write the entire worklog in English. Update `plan.md` while working rather than
reconstructing it only after completion. Record material differences from the
proposal as soon as they become clear.

Implement the requested change and verify it in proportion to its risk. Do not
write implementation findings back into `proposal.md`; the worklog owns all
post-proposal knowledge.

## Finish or Abandon the Work

When implementation succeeds, add an English outcome and a detailed account of
the difficulties and deviations. Treat unexpected findings as more valuable
than routine success, and describe differences from expectations in greater
detail. For every related discussion that has a `discovery.md`, set its status
to `worked` without changing its body. Leave the worklog status as `in-progress`
until the user explicitly invokes the decision workflow.

If the user explicitly abandons the work, record why and set the worklog status
to `abandoned`. Set a related discovery to `dropped` only when the user also
explicitly drops the entire proposal; otherwise preserve its current status.

## Review Structural Difficulties

Identify difficulties that look like flaws in the design or assumptions rather
than one-off accidents. Ask whether the user wants to create a new
`discovery.md`. Do not create one without the user's explicit request; the user
owns the initial discovery text.
