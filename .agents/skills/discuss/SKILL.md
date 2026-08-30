---
name: discuss
description: Examine a discovery selected by discussion number or path, challenge its assumptions, and write Japanese minutes and a proposal only after the user explicitly asks to conclude the discussion.
---

# Discuss a Discovery

Follow `docs/README.md`.

Treat the argument in the user's request as the input. Resolve it in this order:

1. If it is a bare sequence number such as `7` or `0007`, zero-pad it to four
   digits. Find immediate subdirectories of `docs/discussions/` whose names are
   exactly that number or start with that number followed by `-`. Continue only
   when exactly one directory matches; if none or more than one match, stop and
   ask the user how to proceed.
2. Otherwise, accept a repository-relative path to a directory under
   `docs/discussions/` or any file inside one. Resolve a file input to its
   containing discussion directory.

If the input matches neither form, or the resolved directory has no
`discovery.md`, stop and ask the user how to proceed.

## Inspect the Discovery and Existing Context

Read `discovery.md` first. If it lacks frontmatter, infer and add the required
metadata without changing its body.

If the discussion directory contains only a four-digit number, derive a short
English kebab-case description from the discovery and rename the directory to
`NNNN-short-description`. Otherwise preserve the existing directory name. Use
the resulting path for the rest of the workflow.

Read relevant documents under `docs/design/` and `docs/decisions/`, then inspect
earlier discussions with overlapping frontmatter tags. If an earlier discussion
considered and rejected the same option, tell the user before continuing.

## Discuss Before Writing

Do not create `proposal.md` or `minutes.md` during the discussion.

Ask about missing context, present meaningful options the user has not raised,
and challenge questionable assumptions. Do not optimize for agreement. Ask no
more than three questions at once. If an answer remains ambiguous in a way that
affects the proposal, ask again instead of guessing.

Continue until the user explicitly asks to conclude or summarize the proposal.

## Conclude the Discussion

Once the user asks to conclude and there is sufficient agreement:

- Write `minutes.md` in Japanese. Summarize the important points rather than
  transcribing the conversation. Record the questions asked, rejected options,
  and the reasons they were rejected.
- Write `proposal.md` in Japanese. Capture the proposal and its reasoning as
  understood at that time; do not present it as an accepted decision.
- Set the `discovery.md` frontmatter status to `proposed` without changing its
  body.

If the user explicitly drops the discussion instead, write or update
`minutes.md` in Japanese with the reason, do not create a proposal, and set the
discovery status to `dropped`. Never delete the discussion.
