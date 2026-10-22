# Proposed skills configuration

Status: draft for review. GitHub Issues was selected by the project owner. The
default triage labels below are proposed and await the owner's answer.

This configuration sets conventions for Matt Pocock's engineering skills. It does
not select the application architecture.

## Block to append to CLAUDE.md

```markdown
## Agent skills

### Issue tracker

Track issues and specs in `universitysjp/context-bridge` using GitHub Issues. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the five standard triage labels. See `docs/agents/triage-labels.md`.

### Domain docs

Use a single project glossary and ADR directory. See `docs/agents/domain.md`.
```

## docs/agents/issue-tracker.md

```markdown
# Issue tracker: GitHub

Issues and specs live in `universitysjp/context-bridge` on GitHub. Use the `gh` CLI.
Run commands in this checkout, or pass `--repo universitysjp/context-bridge` explicitly.

## Conventions

- Create an issue with `gh issue create --title "..." --body-file <path>`.
- Read an issue with `gh issue view <number> --json number,title,body,labels,comments`.
- List issues with `gh issue list --state open --json number,title,body,labels`.
- Add or remove labels with `gh issue edit <number> --add-label "..."` or `--remove-label "..."`.
- For multiline issue bodies, PR bodies, and comments, write the exact text to a temporary file and use `--body-file`.
- When a skill says "publish to the issue tracker", create a GitHub issue.
- When a skill says "fetch the relevant ticket", read the GitHub issue.

## Pull requests as a triage surface

PRs as a request surface: no.

## Wayfinding operations

- Keep the wayfinding map in a GitHub issue labelled `wayfinder:map`.
- Link child tickets using native GitHub sub-issues when available; otherwise use `Part of #<parent>` in the child and a task list in the parent.
- Represent blockers using native GitHub issue dependencies when available. API requests use the blocker's numeric database ID, not its issue number or node ID.
- If dependencies are unavailable, add `Blocked by: #<number>` to the ticket body.
- A ticket is ready when it is open, unassigned, and all its blockers are closed. Take ready tickets in map order.
- Claim a ticket by assigning it to the driving developer.
- Resolve a ticket by recording its answer, closing it, and adding the decision and issue link to the map.
```

## docs/agents/triage-labels.md

```markdown
# Triage labels

| Skill role | GitHub label | Meaning |
| --- | --- | --- |
| needs-triage | needs-triage | Maintainer needs to evaluate the issue |
| needs-info | needs-info | Waiting for more information |
| ready-for-agent | ready-for-agent | Fully specified and ready for an agent |
| ready-for-human | ready-for-human | Requires human implementation |
| wontfix | wontfix | Will not be actioned |

Use these label strings when a skill names a triage role.
```

## docs/agents/domain.md

```markdown
# Domain documentation

This project uses one shared domain context.

## Before exploring

Read `GLOSSARY.md` at the project root and any relevant decisions in `docs/adr/`.
If these files do not exist, proceed. Create them through `/domain-modeling` as
terms and decisions are resolved during `/grill-with-docs`.

## Vocabulary

Use the glossary's terms in discussions, issue titles, code, and tests. Challenge
ambiguous language during the interview before introducing new terminology.
Keep implementation details out of glossary definitions.

## Decisions

Surface conflicts with an existing ADR explicitly before proposing a change.
Record an ADR when a decision has a meaningful reversal cost, needs context for
future readers, and results from a real tradeoff.
```
