# Namespace и связи с задачами

Wiki Space представляет Wiki-проект сквозного Namespace. Admin хранит
канонический binding; Wiki хранит подтверждённую projection с generation,
operation ID, registry/instance/resource UUID. Чтение и обычная работа active
Space не требуют доступности Admin. Потерянная projection закрывает writes.

## API и данные

Узкий machine owner API: `GET/PUT /api/v1/namespace-resources/wiki_space/{id}`.
Human API: `/api/v1/namespace-contexts`, `/namespace-available-resources`,
`/namespace-stats/{registry}/{namespace}`. Каталоги ограничены page limit/offset.

Wiki владеет document-task link. `POST /api/v1/spaces/{key}/managed-task-links`
принимает typed TaskRef, Document UUID и Revision UUID. Ограниченный Tracker
reader проверяет Task/Namespace; клиентский PAT не пересылается. Сохранённый
повтор возвращает исходную связь до remote validation. GET
`/api/v1/managed-task-links/{tracker_instance}/{task}` читает сохранённые links.
Чтение не вызывает Tracker; отсутствующая projection даёт unavailable.

Migration `202610080001` добавляет local bindings, managed marker, typed dossier
identity и immutable revision links. Уникальность managed dossier:
`(space_id, tracker_instance_id, task_id)`; `task_key` — display snapshot.
Legacy dossier сохраняет `(space_id, task_key)`. Неоднозначный key URL получает
conflict; единственный managed dossier перенаправляется на стабильный TaskRef.
Непроверенные external/legacy links сохраняют прежний статус.

Опубликованная revision managed Space неизменяема; новая публикация создаёт
новую revision. Archive закрывает mutation paths, включая старые API и tables.
Namespace restore сохраняет IDs и независимый archive документа.

## Настройка и поставка

`WIKI_NAMESPACE__INSTANCE_ID`, `REGISTRY_INSTANCE_ID`, `OWNER_SUBJECTS` и
`READER_SUBJECTS` задаются deployment. Tracker reader использует отдельные
`WIKI_NAMESPACE__TRACKER_URL`, `TRACKER_INSTANCE_ID`, `TRACKER_TOKEN_FILE`.
Endpoints фиксированы, redirects отключены, response ограничен 64 KiB и 10 s.
Machine identity проверяется до создания human profile. Verified display name
берётся из Central Auth по subject; email не служит identity или именем.

`VITE_NAMESPACE_ENABLED=true` включает picker и managed task UI только после
установки совместимого cohort. Legacy SDK/skills pins не переписываются.
Fresh source/CI/image/served/UI evidence и backup/rollback проверяются отдельно.

PAT старого Auth без display metadata использует только уже сохранённый active
профиль по exact central subject. Имя, роль и timestamp профиля не меняются;
неизвестный или отключённый профиль отклоняется. Machine classification
предшествует этому lookup, scopes и отзыв credentials продолжают действовать.
