# Standalone container producers до подготовки

Использовать original pinned Base selector вместо implicit Compose build all.
No args — HEAD плюс staged/unstaged/untracked/deleted; явные services задают scope,
--all выбирает все buildable services. Пустой выбор не вызывает verifier/Docker
или UI preparation и не является readiness/artifact proof. Cold bootstrap явный.
Packaging recipes/nginx входят в отдельную карту, unknown inputs расширяют выбор.
Недостоверный ref/неподдерживаемая карта отказывает до подготовки.


SDK/UI pins независимы; mixed single-context build отказывает до подготовки.
Verifier любого выбранного Source должен пройти до Docker. Pins, vendor selector,
application gates/recipes/Compose, toolchains и manual release obligations прежние.
Raw heavy команды используют существующий общий host slot, Root owner не оборачивать.

Scoped проверки: stdlib caller/map units, existing source neighbors, actual Git
fixtures и исходный Root Source route. Builds/install/Docker/WSL/hosted CI/runtime
не выполнять; actual native/cache/resource gates сохраняются на общей вехе.

Final Source suite11 PASS; actual Root Unit с Repository wiki, Component tooling
и пустым NativeDockerContext выполнил только Source/static gates, все PASS.
Десять Git fixture scenarios покрывают clean/docs/test-only, staged/unstaged/
untracked/deleted, packaging, unknown input и invalid HEAD; verifiers/producers
заглушены, fixtures удалены. Ruff и product docs validator PASS.
Нативные build/cache/resource и integrated/manual/live gates этим не подтверждены.

Root Source qualification обнаружила два pre-existing stale guards. Release уже
проверяет ignored generated outputs через no-symlink/nonempty и independent
openapi:check; тест согласован с этим контрактом, tracked lockfile guard сохранён.
Generator hash соответствует reviewed port6c6bc5d/c2649c33: extra read только
объявленного output для подавления идентичной записи. Шесть existing synthetic
Node cases PASS на installed Node24.19.0; это Source proof, не product Node22
или новое native prepared-environment grant. Original Root failure сохранён;
после исправления оба Source gates PASS. Генератор и release workflow не менялись.
