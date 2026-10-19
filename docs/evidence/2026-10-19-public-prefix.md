# Public prefix source qualification, 2026-10-19

The integrated candidate preserves the current product baseline. Its mounted deployment uses the existing shared router/API mechanisms; explicit API origin and explicit empty root overrides remain supported. No credentials, data or installed runtime changed in this qualification.

- Node 22.20.0, pnpm 10.28.1: offline frozen install, TypeScript, lint and format, 252 frontend tests with one worker, and one mounted production build passed.
- Codex IAB loaded the actual built assets under the product prefix using a private read-only fixture with synthetic public configuration. Login rendered at 320/375/1920/2560 CSS pixels without horizontal overflow; mobile SSO buttons measured 44 pixels. Reload and Back/Forward retained the mounted routes.
- This browser check did not authenticate, issue credentials or call an installed product backend. Synthetic public configuration and denied protected endpoints do not prove authenticated SSO or production acceptance.
- Build/source input hashes and local logs are retained in the task qualification packet. Hosted CI is reported separately.

UI pin 8098f29333bb9142b1c82811b2dd498a6d40a67f supplies the public basename option required by this candidate. SDK and skills pins remain independent.
