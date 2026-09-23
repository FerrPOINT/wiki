# Evidence control target QA

Дата: 2026-09-23

## Дефект

На production bundle поля, кнопки, ссылки реестра и действия строк имели
интерактивную высоту 20-36 px. На dense fixture из 20 материалов дефект
воспроизводился во всех 15 сочетаниях пяти viewport и трёх тем: до 117 из 117
видимых controls оказывались меньше платформенного минимума 40 px.

## Изменение

- Поля создания и фильтрации, переключатели типа материала и основные действия
  получили минимальную высоту 40 px.
- Названия материалов и ссылки на документ, задачу и фазу получили полную
  40 px интерактивную область без увеличения текстового размера.
- Icon actions закреплены в размере 40 x 40 px и не сжимаются рядом с длинным
  заголовком на tablet.
- Плотная table/card компоновка, перенос длинных значений и текущая
  responsive-структура сохранены.

## Production candidate QA

Проверено с настоящими Central Auth и Wiki API и детерминированным read-only
dense fixture:

- viewport: `375`, `768`, `1280`, `1920`, `2560`;
- темы: light, gray, dark;
- 15/15 candidate-состояний прошли;
- видимые controls меньше 40 px: 0 (baseline: до 117);
- root overflow: 0;
- serious/critical Axe: 0;
- page errors: 0;
- неожиданные API errors: 0;
- write requests: 0.

Полные измерения находятся в `baseline-report.json` и
`candidate-report.json`. PNG-файлы содержат показательные mobile, desktop и
ultra-wide состояния до и после изменения.

## Automated gates

- Evidence page tests: 13/13;
- frontend suite: 24 файла / 138 тестов;
- TypeScript, ESLint, semantic color lint и production build: green;
- OpenAPI schema и compatibility: green;
- focused Prettier и `git diff --check`: green.
