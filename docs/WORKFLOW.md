# Workflow Integration - Wiki

## 1. Purpose

Wiki does not execute SDLC workflow itself. It stores documents and evidence for each phase executed by `project-workflow`.

## 2. Phase Mapping

| Phase | Expected Wiki Material |
|---|---|
| analysis | requirements, scope, constraints |
| design | architecture note, ADR links |
| implementation | implementation notes, PR links |
| testing | test plan, CI evidence, manual QA notes |
| release | release note, deployment evidence |
| rollback | incident note and recovery evidence |

## 3. Completion Policy

A phase can be marked complete in Wiki when:

- phase dossier exists;
- required document template is present or explicitly skipped;
- required evidence exists for testing/release phases;
- audit event is written.

## 4. Phase Updates

Phase state can be recorded through ordinary API or CLI updates. Event-driven sync from `project-workflow` is deferred until a separate integration scope is approved.

## 5. References

- `docs/AUTOMATION_ARCHITECTURE.md`
- `docs/EVENTS.md`
