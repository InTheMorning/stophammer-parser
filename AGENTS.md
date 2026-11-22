# stophammer-parser Agent Guidelines

A declarative RSS and Podcast XML extraction engine. A rule declares where a
value is in the document, how to transform it, and which field receives it.
No network, no database, no async. The input is one XML string.

Follow the `project-baseline` skill. It holds the shared working rules.

`stophammer/docs/adr/` owns every decision that shapes this crate.
`stophammer/docs/adr/README.md` is the index. musicindex ADR 0086 R-FEED-1
to R-FEED-4 (`~/build/musicindex/docs/adr/`) make this crate the suite
parser for Podcasting 2.0 feeds.

The crate has two users: `stophammer-crawler` and HitIt. The crawler uses
it by path. HitIt uses it as a Git dependency at a fixed revision
(musicindex ADR 0086 R-FEED-1 item 2). A change to the public types or to a
read rule can change the two users, so tell the orchestrator session about
each such change.

## Where The Work Stands

2026-09-25: `stophammer` ADR 0052 task 004 is complete. `IngestFeedData` has
`new_feed_url` from the channel `itunes:new-feed-url`, and `locked` and
`locked_owner` from the channel `podcast:locked`. Three rules in
`src/profile.rs` fill them. `stophammer-crawler` builds against this crate by
path, so the two commits go together.

`stophammer` ADR 0043 is Accepted, and this crate carries it. `FeedField::LastBuildDate` holds `lastBuildDate`, and
no rule lets that element supply a release date. `src/profile.rs:165` is the
rule.

`stophammer` ADR 0093 is Accepted, and this crate carries it. A live item
reads each `podcast:contentLink` child element into a `content_link` link with
a label. `content_link` is the first `href`. It is not the enclosure URL.

`stophammer` ADR 0098 is Accepted, and this crate carries its §1 to §3. Each
payment route and each value time split has the `value_type`, the
`value_method` and the `block_index` of its `podcast:value` block.
`RouteType::Other` holds each recipient `type` that is not `node` or
`lnaddress`. No release holds this change.

musicindex ADR 0086 R-FEED-2 to R-FEED-4 are in this crate. No release holds
them.

- `FeedParser::read` gives `FeedValues` with no required-field check.
  `check::required_fields` is the stophammer check. `FeedParser::parse` is
  `read` followed by that check, so the crawler sees no change.
- Each payment route has `type_text`, `split_text` and `fee_text` adjacent to
  the parsed form. `FeeReading` holds the `fee` rule, and
  `IngestPaymentRoute::fee_warning` gives its warning.
- The parsed form keeps its defaults in `read`: a missing `type` gives
  `RouteType::Node`, and a missing `split` gives 0. The text field is `None`,
  so an app can apply its own rule. This is the stophammer selection of
  R-FEED-2 item 3 for R-FEED-4 item 2.
- `tests/adr0086_feed_read_tests.rs` holds the tests of R-FEED-2 to R-FEED-4.

## What Is True Here

- **The engine keeps the first value.** `src/engine.rs:118` and `:364` hold
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
