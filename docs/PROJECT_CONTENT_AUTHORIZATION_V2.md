# Wiki: допуск внутри транзакции записи

Source candidate. Runtime activation требует общего совместимого cohort;
этот документ не подтверждает его установку.

## Источник полномочий

Identity/session/PAT проверяет Base Auth. Immutable WikiSpaceAdmission содержит
исходные subject, product scopes, локальные role ceilings, SpaceRef/catalog revision
и абсолютный/монотонный срок. Клиентский JSON не является proof. Общий admission
даёт видимость, текущая подтверждённая локальная роль определяет изменение.
Write PAT не требует дополнительного read scope только ради получения identity;
read PAT не превращается в write. Metadata lifecycle и Tracker projection имеют
отдельные зарегистрированные owner ports.

## Граница SQLx

begin_content_write использует существующую SQLx Transaction, без отдельного
протокола, ambient task-local scope или SQL parser. Порядок блокировок:
admission singleton → sorted catalog → Space → актуальный profile → local role.
Scope проверяется после ожидания target row, profile связывается с original
central subject, catalog revision сравнивается с исходным подтверждением.
FOR SHARE сохраняет принятое profile/role решение до commit. Ordinary content
держит shared Space lock и exclusive document lock, без позднего upgrade Space.
Admin metadata/membership берут exclusive Space lock сразу. Два независимых
редактора одного пространства не создают upgrade deadlock. Catalog/Space locks
согласованы с owner lifecycle. Ошибка/конфликт не запускают новый ключ или retry.

Документы, task/phase links, evidence, member management и legacy Space writes
используют общий begin/commit. Commit проверяет исходный deadline после SQL waits,
content и audit writes; истечение вызывает rollback. Новое окно не выпускается.
Canonical document ownership читается у document owner, не из клиентского Space.
Миграции и история публикаций этим guard не переписываются.

## Личные и служебные ресурсы

Upload до привязки — staged attachment его загрузившего пользователя: проверяются
profile/identity и write scope, без выдуманного default Space. Привязка evidence
проверяет uploader и target Space в транзакции. При неудачном upload удаляется
только его новый task-owned файл прежним cleanup. Maintenance удаляет только
подтверждённые просроченные unbound staged записи; это отдельный внутренний порт,
не пользовательский grant на project/Space.

## Проверки и границы

Native fixture использует actual create/update paths и ожидающую SQL-транзакцию:
отзыв редактора до снятия Space row barrier запрещает создание; expiry во время
document row wait откатывает draft/title; свежая разрешённая запись проходит.
Отдельно проверены read-only PAT редактора, обычный читатель и hidden Space.
Все машины, потоки/download, owner completion и полный serving cohort требуют
самостоятельной квалификации; наличие этого helper не доказывает их покрытие.

## Ошибки PostgreSQL

Внутренний SQLx adapter сохраняет SQLSTATE до формирования общего AppError.
42501 даёт403 без private diagnostics,40001/40P01 дают409 concurrent_change,
23505 сохраняет409 duplicate entry. Роль и допуск не расширяются, автоматического
повтора записи нет: caller перечитывает состояние и использует исходный operation
key. Прочие ошибки сохраняют прежнее общее преобразование; внутренние сообщения БД
не выдаются в HTTP. Bootstrap/migration handling и общий
SDK error envelope не изменяются.
