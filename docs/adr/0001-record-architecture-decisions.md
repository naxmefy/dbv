# 0001. Record architecture decisions

Date: 2026-10-23

Status: Accepted

## Context

dbv is an open-source project that will grow over many milestones and attract outside contributors. Decisions such as the crate layout, the driver interface or the on-disk formats shape everything built later. If the reasoning behind them lives only in chat threads or in the maintainer's head, newcomers cannot tell why things are the way they are, and the same debates repeat.

## Decision

We record significant architectural decisions as Architecture Decision Records (ADRs), following the lightweight format popularised by Michael Nygard.

- ADRs live in `docs/adr/` as Markdown files named `NNNN-short-title.md`, numbered sequentially.
- Each ADR follows `docs/adr/template.md`: title, date, status, context, decision, consequences.
- ADRs are never deleted and their decision text is not rewritten after acceptance. To change a decision, write a new ADR and mark the old one `Superseded by [NNNN](...)`.
- `docs/adr/README.md` is the index and is updated in the same pull request that adds an ADR.
- ADRs are proposed through normal pull requests, so they are reviewed like code.

## Consequences

- The history of why the project looks the way it does is preserved in the repository.
- Contributors have one place to check before reopening a settled question.
- Writing an ADR costs a little time per significant decision; small or easily reversible choices do not need one.
