# Backup & Restore

## 1. Что бэкапим

| Компонент | Способ | Частота |
|---|---|---|
| PostgreSQL | `pg_dump` | ежедневно |
| Attachments | `rsync` / object storage replication | ежедневно |
| `.env` | внешний secret manager / encrypted store | при изменении |

## 2. Автоматический бэкап

```bash
./scripts/backup.sh
```

Скрипт делает:

1. `pg_dump -Fc` из compose service `postgres` в `postgres.dump`.
2. Копирует текущий storage directory из service `backend` (`WIKI_STORAGE__DIR`, default `/var/lib/wiki/uploads`) в `attachments.tar`.
3. Добавляет `manifest.env` с датой, DB name/user и storage path.
4. Собирает архив `backups/wiki-YYYYMMDD-HHMMSS.tar.gz`.

Если backend container не запущен, скрипт всё равно создаст database backup, но положит пустой `attachments.tar` и напишет warning. Для полного production backup backend должен быть running или хотя бы существовать с подключенным volume `uploads`.

Ротация выполняется отдельной командой:

```bash
./scripts/cleanup_old_backups.sh 30
```

### Cron

```cron
0 2 * * * cd /opt/dev/wiki && ./scripts/backup.sh >> /var/log/wiki-backup.log 2>&1
```

## 3. Ручной бэкап

```bash
# PostgreSQL
docker compose exec -T postgres pg_dump -U "${POSTGRES_USER:-wiki}" -d "${POSTGRES_DB:-wiki}" -Fc > postgres.dump

# Attachments
docker compose cp backend:"${WIKI_STORAGE__DIR:-/var/lib/wiki/uploads}/." ./attachments-backup
```

## 4. Восстановление

```bash
./scripts/restore.sh ./backups/wiki-YYYYMMDD-HHMMSS.tar.gz
```

Порядок:

1. Скрипт останавливает `backend` и `frontend`, чтобы во время restore не было записей.
2. Скрипт поднимает `postgres`, если он не запущен.
3. Восстановить Postgres:
   ```bash
   docker compose exec -T postgres pg_restore -U "${POSTGRES_USER:-wiki}" -d "${POSTGRES_DB:-wiki}" --clean --if-exists --no-owner < postgres.dump
   ```
4. Очистить attachment volume и восстановить `attachments.tar`.
5. Запустить `backend` и `frontend`.
6. Проверить `/api/v1/health`, затем `/api/v1/health/ready`.

## 5. Point-in-time recovery

- Если включён WAL archiving — восстановление до момента времени.
- Нужен отдельный backup solution (Barman, pgBackRest, WAL-G).

## 6. Object storage backup

Если attachments в S3/MinIO:

- Включить bucket versioning.
- Настроить cross-region replication.

## 7. Проверка бэкапов

- Раз в месяц делать test restore на staging.
- Метрика: `backup_last_success_timestamp`.

Локальный WSL drill для Windows-хоста без Docker Desktop:

```powershell
pwsh -File scripts/backup-restore-smoke-wsl.ps1
```

Команда создаёт две временные PostgreSQL базы и роль в WSL, применяет canonical SQLx migrations к source DB, добавляет контрольный документ/evidence/attachment, делает `pg_dump -Fc` и `attachments.tar`, восстанавливает их в restore DB и сравнивает checksum записи и файла. В конце удаляются только временные базы, роль и временный каталог; с `-KeepArtifacts` архив и распакованные файлы остаются для ручной проверки.

## 8. Disaster recovery

| Сценарий | RTO | RPO | Действия |
|---|---|---|---|
| Потеря данных PG | 1 час | 24 часа | restore из последнего pg_dump |
| Потеря attachments | 30 мин | 24 часа | rsync из бэкапа или S3 |
| Потеря entire host | 4 часа | 24 часа | развернуть на новом хосте из бэкапа |

## 9. References

- `docs/DEPLOYMENT.md`
- `docs/OPS_RUNBOOK.md`
- `docs/MONITORING.md`
- `docs/BACKUP_RESTORE.md`
