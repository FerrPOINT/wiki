# Workflow Documentation DSL - Wiki

## 1. Purpose

Wiki does not run CI/CD pipelines, but it may need a small declarative contract for required documents and evidence by workflow phase. The file keeps parity with the CI/CD `PIPELINE_DSL.md` document and describes a future Wiki workflow-documentation DSL.

The DSL can later be stored in space settings or a template bundle. It defines what knowledge must exist before a phase can be considered documented.

## 2. Target Shape

```yaml
version: 1
workflow: delivery
phases:
  analysis:
    required_documents:
      - type: requirements
        template: requirements
    required_evidence:
      - kind: task_snapshot
  implementation:
    required_documents:
      - type: architecture_decision
    required_evidence:
      - kind: pull_request
      - kind: external_url
  testing:
    required_documents:
      - type: test_report
    required_evidence:
      - kind: ci_artifact
  release:
    required_documents:
      - type: release_note
    required_evidence:
      - kind: deployment
```

## 3. Fields

| Field | Required | Description |
|---|---:|---|
| `version` | yes | Semantic DSL version. Current target is `1` |
| `workflow` | yes | Workflow name from `project-workflow` or manual configuration |
| `phases` | yes | Phase map keyed by stable phase slug |
| `required_documents` | no | Document types and optional template references |
| `required_evidence` | no | Evidence kinds required for phase completeness |
| `optional_documents` | no | Recommended documents |
| `rules` | no | Future validation rules |

Unknown top-level keys are rejected after backend implementation. MVP can start without this DSL and use explicit task/phase links.

## 4. Document Types

Canonical document types:

- `requirements`;
- `architecture_decision`;
- `research_note`;
- `implementation_note`;
- `test_plan`;
- `test_report`;
- `release_note`;
- `incident_note`;
- `manual_note`.

## 5. Evidence Kinds

Canonical evidence kinds:

- `task_snapshot`;
- `pull_request`;
- `review`;
- `ci_artifact`;
- `deployment`;
- `file`;
- `url`;
- `manual_note`.

## 6. Validation Rules

- Phase keys use `^[a-z][a-z0-9_-]{1,63}$`.
- Document and evidence types must be canonical or registered by system admin.
- Template references must exist in the same space or be global templates.
- Required lists must not contain duplicates.
- DSL size is limited to 256 KiB.
- Validation diagnostics must not include secret values or raw private document bodies.

## 7. Completeness Evaluation

A phase dossier is complete when every required document and evidence rule has at least one permitted object linked to the dossier. Completeness is a projection and can be recalculated; it does not mutate published document revisions.

Evaluation returns:

| Status | Meaning |
|---|---|
| `complete` | All required items exist and are visible |
| `missing_required` | At least one required item is absent |
| `blocked` | External source sync failure prevents evaluation |
| `not_configured` | No DSL exists for the project/space |

## 8. Acceptance Criteria

- Workflow phase requirements can be represented without application code changes.
- Dashboard can show missing documents/evidence by phase.
- Approved API/CLI clients can upsert phase pages idempotently.
- Invalid DSL is rejected with line/field diagnostics.
- DSL changes are audited and versioned.

## References

- `docs/WORKFLOW.md`
- `docs/contracts/UI_API_CONTRACT.md`
