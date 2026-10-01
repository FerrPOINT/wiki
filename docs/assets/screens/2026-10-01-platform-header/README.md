# Wiki Header: live evidence

Production nginx и реальные Wiki/Central Auth API в собственном QA-проекте,
без mock API. Секреты и auth traces отсутствуют. PNG full-page, просмотрены
после capture; остальные временные screenshots не публикуются.

- [375 px](spaces-dark-375.png): Header 60 px; touch targets 44 px,
  создание документа в mobile drawer.
- [1920 px](spaces-dark-1920.png): полноширинный Header над nav-only sidebar,
  создание документа в actions.
- [2560 px](spaces-dark-2560.png): тот же Header, work-area без искусственного
  сужения, sidebar 264 px.
- [Меню сервисов 375 px](services-menu-dark-375.png): непрозрачная поверхность,
  шесть healthy UI в продуктовом порядке, текущий Wiki отмечен;
  Central Auth, Java Agent и Pulse отсутствуют.

[Измерения](results.json) содержат 99 Header сочетаний: три страницы,
три темы и 11 ширин 320-2560 px. Keyboard/touch/outside/Escape/focus,
drawer navigation/create/desktop resize, аккаунт и central logout/re-entry
проверяются на живом стенде. API writes/errors во время Header-навигации
должны быть пусты. [Метаданные](metadata.json) фиксируют source/image/config
fingerprints и границы расширенного непрерывного gate.

Рабочий `sdlc-demo` и его volumes не менялись. Это компонентная приёмка Wiki,
не OCI revision attestation и не окончательный релиз всех продуктов.
