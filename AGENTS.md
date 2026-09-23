# stophammer-parser Agent Guidelines

A declarative RSS and Podcast XML extraction engine. A rule declares where a
value is in the document, how to transform it, and which field receives it.
No network, no database, no async. The input is one XML string.

Follow the `project-baseline` skill. It holds the shared working rules.

`stophammer/docs/adr/` owns every decision that shapes this crate.
`stophammer/docs/adr/README.md` is the index.

## Where The Work Stands

2026-09-23: no change is in progress here. `stophammer` ADR 0043 is Accepted,
and this crate carries it. `FeedField::LastBuildDate` holds `lastBuildDate`, and
no rule lets that element supply a release date. `src/profile.rs:183` is the
rule.

## What Is True Here

- **The engine keeps the first value.** `src/engine.rs:98` and `:364` hold
  `if ... is_set(field) { continue; }`. A later rule is a fallback and cannot
  replace an earlier value. Rule sequence is meaning, not style.
- **Add a rule, not a branch.** `src/profile.rs` declares the rule set.
  `src/engine.rs` applies it. A new element needs a rule.
- **Keep the source.** Never discard a raw value because a heuristic prefers
  another. `podcast_namespace` keeps the tags with no typed field yet.
- Lints are `[lints.clippy] pedantic = "deny"` and nothing more.
- Tests live in `tests/`, with samples in `tests/fixtures/`.

## Commits

`git@github.com:InTheMorning/stophammer-parser.git`. `stophammer-crawler`
depends on this crate by path, so run its gate too.
