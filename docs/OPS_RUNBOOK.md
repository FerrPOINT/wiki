# Operations Runbook — Wiki

## 1. Overview

Пошаговые инструкции для типовых операций production-инстанса: deploy, rollback, backup, restore, incident response.

## 2. Daily Checks

```bash
docker compose ps
docker compose logs --tail 100 backend
curl -f https://wiki.example.com/api/v1/health
curl -f https://wiki.example.com/metrics | grep up
```

## 3. Deploy New Version

```bash
cd /opt/wiki
git fetch origin
git checkout main
git pull origin main
docker compose build
docker compose up -d
DATABASE_URL=postgres://... sqlx migrate run --source backend/migrations
docker compose ps
```

## 4. Rollback

```bash
# Revert code
git log --oneline -20
git revert <bad-commit>
docker compose build
docker compose up -d

# DB rollback: apply compensating migration if the release changed schema
```

## 5. Backup

```bash
./scripts/backup.sh
# Verify archive in ./backups
ls -lh ./backups/wiki-*.tar.gz
```

## 6. Restore

```bash
./scripts/restore.sh ./backups/wiki-YYYYMMDD-HHMMSS.tar.gz

# Restart
docker compose up -d backend frontend
curl -sf http://localhost:3456/api/v1/health
curl -sf http://localhost:3456/api/v1/health/ready
```

## 7. Scaling API

```bash
docker compose up -d --scale backend=3
```

## 8. High CPU / Memory

1. Check `top` / `docker stats`.
2. Review slow query log.
3. Restart affected container.
4. Enable rate limit if DDoS suspected.

## 9. DB Connection Pool Exhaustion

```sql
SELECT count(*), state FROM pg_stat_activity GROUP BY state;
```

Mitigation:

- Restart API pods.
- Increase pool size temporarily.
- Kill long-running queries.

## 10. Disk Full

```bash
df -h
docker system df
docker image prune -a
./scripts/cleanup_old_backups.sh
```

## 11. Incident Contacts

- Primary operator: project owner.
- Alert channel: configured by deployment environment.
- Escalation: hosting provider and database administrator.

## 12. Post-Mortem

After every SEV-1/SEV-2 incident:

1. Timeline.
2. Root cause.
3. Impact.
4. Remediation.
5. Preventive actions.

## References

- `docs/DEPLOYMENT.md`
- `docs/MONITORING.md`
- `docs/SECURITY.md`
- `docs/MIGRATIONS.md`
