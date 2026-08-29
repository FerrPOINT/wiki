# Reports - Wiki

## 1. Purpose

Reports are not part of the MVP core. Wiki MVP has no reports route and no report API group; backend report projections should wait until base Wiki flows are implemented.

## 2. Deferred Reports

| Report | Description |
|---|---|
| Documentation summary | Count documents by space/type/status |
| Evidence summary | Count URL/file evidence by space/task/phase |
| Publication activity | Revisions published over time |

## 3. Metrics

- documents published per week;
- evidence records per week;
- archived documents;
- active editors.

## 4. UI

No reports UI is shipped in MVP. Any future page must be approved as a separate product scope and must not introduce required document/material policies retroactively.

## 5. Report Inputs

| Input | Source |
|---|---|
| Published document revisions | Wiki documents |
| Evidence records | Wiki evidence |
| Spaces and members | Wiki spaces |

## 6. Report Outputs

- document counts by space;
- evidence counts by owner;
- latest publication activity.

## 7. Acceptance Criteria

- Report numbers are permission-filtered.
- Report implementation is not required for MVP.
- Any future report uses only existing Wiki data unless a separate requirement is approved.
