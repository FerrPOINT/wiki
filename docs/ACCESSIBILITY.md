# Accessibility - Wiki

## 1. Target

Wiki UI should meet WCAG 2.2 AA for primary workflows.

## 2. Requirements

- Keyboard navigation for app shell, document tree, editor controls and dialogs.
- Visible focus states.
- Sufficient color contrast for text, icons and badges.
- Form inputs have labels or accessible names.
- Icon-only buttons have `aria-label`.
- Document headings preserve semantic hierarchy.
- Toasts and validation messages are announced to assistive technology.
- Tables/lists expose useful labels and row actions.

## 3. Critical Screens

- Login/register.
- Dashboard.
- Spaces/document tree.
- Document editor/viewer.
- Task dossier.
- Phase dossier.
- Search.
- Admin/settings.

## 4. Verification

- Component tests check labels and roles.
- Playwright smoke checks keyboard access to main navigation.
- Manual review on mobile and desktop.
