# Dev SSO собственного workspace

Helper задаёт `VITE_AUTH_CLIENT_ID=wiki-dev`, но общий `ssoConfig` содержит
жёсткий `wiki`. Auth отклоняет Dev callback этого клиента.

Использовать существующую переменную в общем `ssoConfig` для входа, callback,
восстановления сессии и выхода. Default `wiki`, issuer, API, роли и данные
сохраняются. API proxy уже учитывает конфигурацию workspace.

Для native Windows checkout закрепить LF текстовых файлов frontend через Git
attributes: Prettier требует LF, а `core.autocrlf=true` без правила создаёт CRLF.
Проверить экспорт файлов с этой настройкой Git и неизменность их содержимого.

Проверить default и Dev client на реальном вызове восстановления, Dev client
на logout, frontend tests/lint/typecheck/build, OpenAPI drift/compatibility и
переход к форме собственного Auth в Chromium, Firefox и WebKit. Сохранить
скриншоты 375/1920/2560. Этот сценарий не подтверждает полный SSO lifecycle.
