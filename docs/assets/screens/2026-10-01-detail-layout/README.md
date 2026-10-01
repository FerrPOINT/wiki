# Wiki Detail Geometry Evidence

Captured on 2026-10-01 (Europe/Moscow) from real Central Auth/Wiki APIs and a
production nginx image in isolated QA Compose. No intercepted APIs or mock
responses. The test created its own space, document, two revisions and linked
evidence. Only its document and space were archived through supported APIs;
evidence/history remain in that isolated space until QA-project cleanup.

- [Published document and context, dark, 1920 px](document-read-dark-1920.png)
- [Draft editor before context, light, 375 px](document-edit-light-375.png)
- [Task documents and phase index, light, 1024 px](task-light-1024.png)
- [Phase documents/evidence as equal working areas, gray, 2560 px](phase-gray-2560.png)

The browser measured actual rail width (320 px), gap, placement, stacking and
reading width (maximum 760 px), not just mode markers. The four detail states
passed 9 widths x 3 themes (108 combinations), including 1023/1024 and 1279/1280
boundaries. Revision dialogs passed touch/keyboard, Escape and focus return;
document links reached the real task/phase in the non-default fixture space.
Opening read/edit views and snapshots issued no API mutations.

The same continuous run repeated 144 main-route/theme/viewport combinations:
2 tests passed without retries in 3.5 minutes. Both matrices passed page overflow,
console/network and serious/critical axe checks. Full-page captures were opened
and inspected; image height can exceed viewport height.

[results.json](results.json) records the image/config identity and source-content
fingerprints of the patch before commit. It is not an OCI source revision
attestation, an updated user deployment, or the final platform-wide release gate.
Header ownership and the canonical README gallery remain separate audit items.
