# Issue tracker: GitHub

Issues and specs live in `universitysjp/coderune` on GitHub. Use the `gh` CLI.
Run commands in this checkout, or pass `--repo universitysjp/coderune` explicitly.

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
