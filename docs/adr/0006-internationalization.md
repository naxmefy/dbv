# 0006. Internationalization

Date: 2026-10-23

Status: Accepted

## Context

dbv is a public project and users will expect to read the interface in their own language eventually. Retrofitting translation into hard-coded strings is painful, so the mechanism must exist from the first screen, even though only English ships initially. Database content also contains scripts such as CJK that affect layout.

## Decision

- All user-facing UI strings go through `rust-i18n` using the `t!()` macro. Only the `en` locale ships in M1; other languages are welcome as community contributions.
- Machine-readable CLI output (from M2: JSON keys, error codes, exit codes) is never translated, so scripts can rely on it.
- Text widths are computed with `unicode-width`, so wide characters such as CJK align correctly in grids and truncation.

## Consequences

- Adding a language later means adding a locale file, not touching code.
- Contributors must not hard-code UI strings; reviewers check this.
- Stable, untranslated CLI output keeps dbv usable in automation.
- Free-text messages that come from databases are shown as received and are not translated.
