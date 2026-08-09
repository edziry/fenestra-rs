# Runtime oracle V1 reduction contract

Status: complete locally
Work unit: WU-0005
Branch: `test/runtime-oracles`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-09

## Input and budget

Reducer input is a structurally and semantically valid failing `GeneratedCaseV1`
plus its exact `ReplayFailureV1` and fault. The replay failure retains the
fingerprint and its transaction and optional operation IDs. Every candidate
starts from a fresh registered fixture. One evaluation means one replay
attempt; invalid, duplicate, nonmatching, and accepted candidates consume the
budget. A transform that is not strictly smaller is skipped without consuming
budget. No evaluated-case cache or unbounded deduplication set is retained.

`ReducerConfigV1.max_evaluations` is canonical `u32` in `1..=4096`; the known
artifact uses `4096`. If the final permitted evaluation reproduces the failure,
its candidate is accepted before returning `budget-exhausted`; if it does not,
the current case is retained with the same status. `fixed-point` requires a
complete pass through every transform below with no accepted candidate.

## Search order

At each restart:

1. try contiguous transaction removal with block lengths from current count
   down to one and start positions from zero upward;
2. try one-operation removal in current transaction position then operation
   position order, removing an empty transaction;
3. for each insert or move in operation order, try every smaller index from
   zero upward;
4. for each insert, move, update, or remove in operation order, try each smaller
   explicit key field from zero upward only in that record; keys inside owner
   path segments are never simplified and dependent records are not rewritten;
5. for each `ScalarI32` operand in operation order, try `0`, then `1`, then
   `-1`, skipping its current value and duplicate candidates.

Accept the first strictly smaller candidate that is semantically valid and
reproduces the exact fingerprint with no earlier different failure, then
restart at step 1. Removing a fault target, breaking a base-snapshot target,
producing another failure, or changing the first failure's retained transaction
or optional operation ID is a rejected candidate. Removing prefix transactions
may change its replay position without changing those IDs.

## Metric

The metric is `(operation_count, canonical_case_byte_count, operand_magnitude)`
and compares lexicographically. Magnitude is a checked `u128` sum of every
explicit key, index, and absolute `ScalarI32` value; `i32::MIN` is widened before
absolute value. At most 512 such fields cannot overflow `u128`, but the check
remains explicit. Metric bytes encode only case records, not envelope, trace, or
reducer metadata. Artifact-local IDs and keys inside path segments are not
metric operands.

## Reproduction and completion

Artifact verification always reduces the original again and requires the exact
minimized case, completion status, and used-evaluation count. For
`budget-exhausted`, that deterministic rerun is the complete claim; no
fixed-point or idempotence check follows.

For `fixed-point`, reducing the minimized result again must return the same case
and accept zero transforms. Its second-run evaluation count may differ and is
not part of case equality. The known committed case must finish fixed-point
before the budget, strictly reduce from the original, and preserve the exact
primary-fragment keyed-order fingerprint.

This reducer claims only a fixed point under these V1 transforms and order. It
does not claim global, semantic, or academic minimality.
