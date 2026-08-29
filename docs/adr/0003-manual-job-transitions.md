# ADR-0003: Review And Phase Transitions Are Deferred

## Status

Deferred

## Context

Wiki MVP stores documents, revisions, task links, phase links and evidence. It does not own workflow state and does not need document approval chains to be useful as a base Wiki.

Adding review transitions, phase completeness rules or approval commands now would expand the product beyond the agreed MVP.

## Decision

Do not implement review state machines, approval commands or phase completion transitions in MVP.

MVP supports only:

- draft update;
- publish revision;
- archive document;
- link document to task key;
- link document/evidence to phase key;
- add evidence.

## Consequences

- Published revision history remains simple.
- Phase pages show linked materials, not completion verdicts.
- External workflow systems remain source of truth for phase state.
- Review/approval can be added later only with a separate requirement.

## References

- `docs/PRODUCT_REQUIREMENTS.md`
- `docs/API.md`
- `docs/WORKFLOW.md`
