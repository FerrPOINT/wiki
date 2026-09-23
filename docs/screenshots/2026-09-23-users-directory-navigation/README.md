# Users directory navigation QA

Дата проверки: 2026-09-23.

## Область проверки

- `/users` с реальным Central Auth, Wiki API и production frontend bundle;
- поиск по имени, username и email, direct URL и reload;
- пагинация, browser Back/Forward и канонизация query parameters;
- handoff к учётным записям Admin Panel;
- controlled `503`, retry и сохранение поискового контекста;
- dense-каталог, responsive, три темы, accessibility и размеры целей;
- runtime, console, network errors и отсутствие прикладных write requests.

Вход и каталог из 33 профилей проверены на реальных Central Auth и Wiki API.
Для детерминированной проверки каталога из 240 профилей использовался только
read-only browser fixture ответа `GET /api/v1/users`. Запросы изменения данных
Wiki не выполнялись.

## Исходное поведение

- поисковая строка хранилась только в React state и очищалась после reload;
- direct URL и Back/Forward не восстанавливали поиск или страницу;
- каталог из 240 профилей одновременно монтировал 240 mobile-строк и 240 строк
  desktop-таблицы: 2323 DOM-узла;
- высота dense-страницы достигала 17 811 px на 375 px и 9156 px на 1920 px;
- поле поиска и handoff-кнопка имели высоту 36 px.

## Реализованный контракт

- `q` хранит нормализованный поиск, `page` — текущую страницу;
- direct URL, reload и Back/Forward восстанавливают каталог;
- пустые, повторяющиеся и невалидные параметры канонизируются через replace;
- неизвестные соседние query parameters сохраняются;
- изменение поиска возвращает первую страницу без засорения browser history;
- каталог показывает по 25 профилей и точный диапазон текущей страницы;
- счётчик различает общее число профилей и результат фильтрации;
- поиск, handoff и pagination controls имеют цель не меньше 40 px;
- управление учётными записями остаётся в Admin Panel.

## Результат

- real Central Auth/API: 33 профиля, первая страница 25 строк;
- direct URL/reload/Back/Forward/canonicalization/Admin handoff: пройдены;
- controlled `503` и retry без потери `q`: пройдены;
- production candidate: 393 DOM-узла и 1265 px высоты в real desktop flow;
- визуальная матрица: 18/18 состояний, 2 страницы x 3 темы x 3 ширины;
- ширины: 375, 1920 и 2560 px;
- dense candidate: не более 401 DOM-узла и 2180 px высоты;
- horizontal overflow: 0;
- интерактивные цели меньше 40 px: 0;
- serious/critical Axe violations: 0;
- page errors и неожиданные console/HTTP errors: 0;
- прикладные write requests: 0.

Полный машинный результат находится в `qa-summary.json`. Сохранены все 18
screenshots первой и последней страниц каталога.
