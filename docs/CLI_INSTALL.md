# Установка CLI Wiki

Принятая локальная поставка Linux x86_64/WSL от 2026-10-03. Backend sdlc1 обновлён;
live acceptance и 15 минут наблюдения пройдены. [Матрица и откат](CLI_VALIDATION.md).

## Артефакт и требования

Архив `wiki-cli-0.2.0-191b449-x86_64-linux-gnu.tar.gz`; SHA-256:
`fa1d08017a853d2bf4721cfd66ad639ccfbcefca215fa7eae8d1d5aa022ebe89`.
Binary SHA-256: `56367daa2a3943bcdeb88fe9d301bec4f5dbb25bce12b6fc23540a5c7ee77587`.
Code source `191b449bed1c7dc4e594ca5681b7509b45961415`; Base `9408802dfa978cba2f67162a49adca6f65851b01`.
Package `wiki-cli` version `0.2.0`, Rust 1.88.0, locked release build.
Версии/tags/public release не менялись. В архиве только binary, README.md,
SHA256SUMS; credentials/configuration/keys/data отсутствуют.

GNU/Linux x86_64, glibc >= 2.34; OpenSSL 3
(libssl.so.3/libcrypto.so.3) требуется.
Ubuntu 24.04 WSL и Debian 12: outer/inner checksum, отдельный installation prefix,
help, runtime libraries и authenticated read всех трёх API sdlc1 — PASS.
Windows используйте через WSL; native Windows/macOS/musl не поставляются.

## Установка

Получите архив и общий SHA256SUMS из локального комплекта; сначала сверьте outer
checksum, затем установите в новый prefix. Пример не перезаписывает предыдущую
установку и не меняет shell profiles:

```bash
sha256sum -c SHA256SUMS
stage=$(mktemp -d)
tar -xzf wiki-cli-0.2.0-191b449-x86_64-linux-gnu.tar.gz -C "$stage"
(cd "$stage" && sha256sum -c SHA256SUMS)
prefix="$HOME/.local/share/sdlc-cli/delivery-20261003"
mkdir -p "$prefix/bin"
install -m 755 "$stage/wiki" "$prefix/bin/wiki"
"$prefix/bin/wiki" --help
export PATH="$prefix/bin:$PATH"
```

## Подключение и откат

Передайте PAT через SDLC_API_TOKEN или продуктовую переменную из [CLI.md](CLI.md);
не помещайте token в аргументы/историю shell. Read scope не отменяет серверную
авторизацию. URL sdlc1 задаётся явно:

```bash
wiki --api-url http://127.0.0.1:7731/api/v1 --output json space list
```

Для отката CLI используйте сохранённый предыдущий binary/prefix и configuration.
Обновление CLI само не меняет backend. Backend откатывается отдельно по защищённому
rollout manifest на точные прежние pins; автоматический restore данных запрещён.
Известные ограничения старых images, Wiki legacy replay и фактические QA/live
границы перечислены в [CLI_VALIDATION.md](CLI_VALIDATION.md). Production runner/
внешний deploy не сертифицируются. Исторические candidates/checksums сохранены
в предыдущих git revisions и локальных доказательствах, не подменены этой поставкой.

## References

- [CLI](CLI.md)
- [CLI validation](CLI_VALIDATION.md)
- [Base integration](BASE_INTEGRATION.md)
