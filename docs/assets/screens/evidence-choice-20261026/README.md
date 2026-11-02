# Тип материала: общий RadioGroup

[Клавиатурный выбор](radio-group-gray-375.png) показывает настоящий RadioGroup
Base с теми же label/value и segmented appearance, которые использует
EvidencePage. Это компонентная source-сцена с локальным состоянием, а не
полная страница Wiki или приёмка её backend.

IAB: light/dark/gray, native iframe 320/375/1920/2560, отсутствие overflow,
40/44 px label targets и выбор стрелкой. Страница EvidencePage отдельно
проверена 16 unit cases: оба payload, сохранение URL/названия при смене типа,
отсутствие записи от выбора, disabled во время pending, сохранение выбранного
файла после ошибки и возможность повторно выбрать файл после успеха.

UI producer остаётся `c2ca61094a904706fceeda8eb8f3b3cfd2e7cdfc`.
Source hashes собственного QA host сверены, API-запросов и Auth не было;
сервер и вкладка закрыты. Полные страницы, реальный zoom 200% и
authenticated-приёмка остаются отдельным gate.
