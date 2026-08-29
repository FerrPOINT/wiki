# SLO - Wiki

## 1. Service Level Objectives

| Area | Objective |
|---|---|
| API availability | 99.9% monthly for single-instance deployment |
| Common read latency | P95 < 200 ms |
| Document open latency | P95 < 250 ms |
| Search latency | P95 < 500 ms for 100k indexed records |
| Upload metadata latency | P95 < 300 ms excluding file transfer time |
| Background indexing lag | P95 < 60 seconds |
| Restore point objective | <= 24 hours |
| Restore time objective | <= 4 hours |

## 2. Error Budget

Monthly error budget for 99.9% availability is about 43 minutes.

## 3. User-visible SLI

- successful document opens;
- successful publish operations;
- successful evidence ingestion;
- successful searches;
- attachment download success.
