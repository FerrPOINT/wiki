# Wiki: семантика и геометрия detail-страниц

## Контекст

Wiki-часть A04 общего аудита: маркер PageFrame не создаёт split, документ
ограничен целиком режимом reading, а три feature grids расходятся с общим rail.
Используем согласованный платформенный стандарт без новых API или продуктовых
исключений. Header ownership и README gallery остаются отдельными задачами.

## Route Mapping

| Маршрут | Режим | Основная область | Контекст |
| --- | --- | --- | --- |
| `/documents/:documentId` | `detail-with-aside` | опубликованный текст или редактор | связанные объекты, ревизии и положение в дереве |
| `/tasks/:taskKey` | `detail-with-aside` | документы и привязка | индекс фаз материалов |
| `/phases/:phaseId` | `wide` | документы и материалы как две рабочие области | отдельного rail нет |
| `/documents/new`, `/settings` | `reading` | форма или настройки | нет; без изменений |

## Реализация

- Документ и задача используют существующий `page-split` из Base: 320 px
  secondary от 1024 px, ниже secondary идёт после primary в DOM и визуально.
- Текст документа использует общий `page-readable` внутри fluid primary:
  максимум 760 px не ограничивает всю страницу и не применяется к rail.
- У фазы остаётся двухколоночная wide grid от 1280 px; две основные области
  не выдаются за metadata/actions sidebar и не сжимаются до 320 px.
- `min-width: 0`, переносы и существующий labelled/keyboard code/table scroll
  сохраняются. Локальные query/mutation/permission и backend-контракты не меняются.
- Контекстные ссылки документа передают его `space`; найденная потеря контекста
  исправляется и проверяется на отдельном QA-пространстве, не только default.

## Проверки

- Unit: route-to-mode mapping, именованные landmarks и порядок контента.
- Frozen install/codegen, OpenAPI check/compat, lint/semantic, typecheck, полный
  frontend unit и production umbrella build; docs validators и diff check.
- No-mock QA на изолированном стенде: собственные QA space/document/evidence,
  фактические rail/gap/position и wide columns на граничных ширинах, три темы,
  full-page screenshots, keyboard revisions и чтение/правка. Штатная архивация
  только собственных записей; volumes рабочего стенда не затрагиваются.
- Повторить существующую live route/theme/viewport матрицу Wiki с axe на том же
  image. Публиковать выбранные кадры и provenance, не секреты или raw auth logs.
