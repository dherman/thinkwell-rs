# Example applications

The examples teach how ordinary Rust computation and typed agent requests fit
together. They use the public library as standalone unpublished Cargo packages,
with path dependencies and separate lockfiles. Their native and analysis
libraries stay outside the Thinkwell library's dependency graph. CI explicitly
checks each package; live agent demonstrations remain opt-in.

`R-easy-agent-programming`, `R-code-prompt-ergonomics`, `R-progressive-control`,
and `R-idiomatic-reimplementation` govern their teaching purpose. Each example
keeps Plan calls visible in its main workflow, uses original small fixtures and
walkthroughs, and distinguishes application-owned facts from generated prose.
These application rules do not extend library guarantees.

## Meeting scheduler

The scheduler reads a fixed roster of stable IDs, names, and natural-language
availability. A `Plan<ParsedAvailability>` extracts weekday/hour windows. Rust
prints and validates those windows, runs Z3 on a blocking task, and independently
rechecks any returned meeting slot. A `Plan<String>` in the same session explains
the outcome. The canonical solver status and time remain visible if explanation
fails. No application tool is needed: Rust invokes the solver between prompts.

Application rules are one shared timezone, one notional Monday–Friday week,
whole-hour starts, and one-hour meetings wholly inside 09:00–17:00 and every
attendee's window. Rosters must be nonempty with unique IDs; extraction must
contain each input ID exactly once and no unknown IDs. The original roster owns
names. Windows require `0 <= start < end <= 24`. Empty availability means no
feasible slot. The solver query is bounded to ten seconds; Unknown, including
timeout, is inconclusive, and model-reading failures are errors. Any valid slot
is acceptable; no earliest-slot promise applies.

The example uses the vendored Z3 source build through the pinned example lockfile;
its native compiler and CMake setup belongs in its walkthrough. Schemas use the
library's ordinary nonrecursive inlining; roster and range checks remain Rust
application code. Z3 checks the extracted constraints, not the model's interpretation
of human intent. Calendars, booking, dates/DST, multiple timezones, fractional
hours, and variable durations are outside the example.

## Sentiment analysis

The sentiment example reads a fixed short English feedback document, constructs
one VADER analyzer, and builds a `Plan<Analysis>` with JSON context and an inline
`analyze_sentiment` tool. Its callback borrows the analyzer and a mutable measurement
log. `run(&agent)` owns the temporary session; the agent selects passages, invokes
the tool, and returns a tone, passage summaries, and recommendation. The explicit
catalog/binding API remains available when an application needs persistent history.

The callback rejects empty or non-exact excerpts, scores valid passages, and logs
the actual inputs and measurements. At least one successful scoring call is
required before presenting a report as tool-backed. Local scores are printed
separately from generated interpretation, remain accessible after the turn, and
survive final-answer failure. The final answer type does not duplicate the scores.

`vader_sentiment` 0.1.1 supplies compound, positive, negative, and neutral f64
measurements. The wrapper checks required keys, finite values, and valid ranges.
Compound lies in [-1, 1] and is not a probability; empty input yields zeros.
Scoring is synchronous on a small fixture. Accuracy, sarcasm recognition,
multilingual support, large-document scheduling, arbitrary-file input, and a
production analysis service are outside the example. VADER remains a dependency
with its notices and research attribution in the walkthrough.

## JavaScript unminifier

The unminifier is a readable teaching demo with happy-path assertions. It reads a
JavaScript module, discovers private top-level functions with Oxc 0.159.0, asks
for typed name suggestions in sequential batches, and applies deterministic
renames. The main workflow shows discovery, `suggest_names`, and renaming.
A reusable plan carries source through `Plan::code`; each batch adds JSON
candidates and executes with fresh history. Candidate identity and ordering,
not response order, determine edits.

The helper edits declaration and resolved-reference byte ranges in reverse order,
preserving untouched source, comments, and whitespace. It reserves existing names,
adds numeric suffixes for collisions, leaves named exports and unrelated shadowed
bindings alone, and expands shorthand properties to preserve their keys. A decoded
`__proto__` shorthand key uses computed-key syntax to preserve a data property.
Oxc is a strict parser; no permissive parsing contract applies.

The demo assumes ordinary lexical JavaScript and sensible identifiers. It does not
handle every export, eval, reflection, or unusual assignment hazard. Renaming can
change observable function names and stack traces. Output uses a new file and
protects an existing destination; a failed write can leave a partial new file.
Atomic output, equivalence proofs, exhaustive validation, retry policies, size
limits, UMD conversion, and parallel batches are outside its teaching scope.
