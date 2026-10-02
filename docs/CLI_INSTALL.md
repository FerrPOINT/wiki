# Установка CLI Wiki

Локальный candidate подготовлен 2026-10-02 из main `4a85f6a7c1de82e0869528788710da258c355ffe`, Base `c083783a37791e277db796361203884b87828a7d`, Rust 1.88.0, `cargo build --locked --release -p wiki-cli`. Package version: `0.2.0`; версию продукта не меняли, Git tags и публичный release не создавали.

## Артефакт и требования

Архив: `wiki-cli-0.2.0-4a85f6a-x86_64-linux-gnu.tar.gz`. SHA-256 архива: `c473ee67c16e7104dd035ac981cf2f294a7049378bd2453b5814d4d5ff5cc482`. Внутри только `wiki`, `README.md` и `SHA256SUMS`; credentials и данные стенда не включены.

GNU/Linux x86_64, glibc >= 2.34, OpenSSL 3 (libssl.so.3 и libcrypto.so.3). Живой API-прогон выполнен в Ubuntu 24.04 WSL; запуск/справка дополнительно проверены в Debian 12. Native Windows/macOS и musl-сборки не подготовлены. На Windows используйте WSL. У CLI пока нет `--version`; source/package version и checksum берутся из manifest.

## Установка в Linux / WSL

После получения архива сверьте его SHA-256 с manifest, затем:

```bash
workdir=$(mktemp -d)
tar -xzf wiki-cli-0.2.0-4a85f6a-x86_64-linux-gnu.tar.gz -C "$workdir"
(cd "$workdir" && sha256sum -c SHA256SUMS)
mkdir -p "$HOME/.local/bin"
install -m 755 "$workdir/wiki" "$HOME/.local/bin/wiki"
export PATH="$HOME/.local/bin:$PATH"
wiki --help
```

При существующей установке сначала сохраните предыдущий binary для отката. Для проверки этой поставки использован отдельный prefix `/root/.local/share/sdlc-cli/cli-20261002/bin`; прежние команды не перезаписывались.

## Подключение к sdlc1

```bash
export WIKI_API_URL=http://127.0.0.1:7731/api/v1
wiki space list
```

Передайте PAT через `SDLC_API_TOKEN` либо продуктовую переменную из [CLI.md](CLI.md). Read-команды требуют `wiki:read`, mutations — соответствующий `wiki:write`; scopes не отменяют серверную авторизацию. Значение token не помещайте в аргументы, историю shell, manifest или release notes. URL здесь относится к sdlc1; для другого стенда задайте его явно.

## Статус приёмки и откат

Это candidate: полная приёмка против принятого runtime не завершена. Ограничения и результаты — [CLI_VALIDATION.md](CLI_VALIDATION.md). Обновление CLI не обновляет backend; рабочие runtime images/pins в этой проверке не заменялись. После согласованного обновления backend повторить блокирующие сценарии; затем решать о tags, версиях и публикации release. Для отката верните предыдущий binary и конфигурацию URL.

## References

- [CLI](CLI.md)
- [CLI validation](CLI_VALIDATION.md)
- [Base integration](BASE_INTEGRATION.md)
