# Code Style — Wiki

## 1. Overview

Единые соглашения по коду для backend (Rust) и frontend (TypeScript/React). Цель — читаемость, минимум дискуссий в PR, консистентность.

## 2. General

- Код и комментарии — русский или english? **English для кода и комментариев**, русский для пользовательских строк и документации.
- Line ending: LF.
- Encoding: UTF-8.
- Max line length: 100.
- Indent: 2 spaces (frontend), 4 spaces (Rust).

## 3. Rust

### 3.1 Format

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

### 3.2 Naming

| Type | Convention | Example |
|------|------------|---------|
| Modules / crates | snake_case | `document_service` |
| Types / traits | PascalCase | `DocumentService` |
| Functions / methods | snake_case | `publish_document` |
| Variables | snake_case | `document_id` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_PAGE_SIZE` |
| Error enum variants | PascalCase | `NotFound` |
| Generic params | single uppercase | `T`, `E`, `Ctx` |

### 3.3 Imports

```rust
// 1. std
use std::sync::Arc;

// 2. external crates
use axum::extract::State;
use sqlx::PgPool;

// 3. internal crates
use app::WikiAppContext;
use domain::wiki::Document;
```

### 3.4 Error Handling

- Использовать `?` оператор.
- Не использовать `.unwrap()` / `.expect()` в production коде.
- В тестах `.unwrap()` допустим.

### 3.5 Async

- Все IO-bound операции — async.
- `tokio::spawn` только для background tasks.

### 3.6 Comments

```rust
/// Publishes a document revision in the given space.
///
/// # Errors
/// Returns `DomainError::NotFound` if document does not exist.
pub async fn publish_document(...) -> Result<Document, AppError> {
    // ...
}
```

### 3.7 Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publishes_document() {
        // arrange
        let ctx = test_context().await;
        let cmd = create_command();

        // act
        let document = ctx.document_service.publish(cmd).await.unwrap();

        // assert
        assert_eq!(document.title, "Release notes");
    }
}
```

## 4. TypeScript / React

### 4.1 Format

```bash
pnpm prettier --write .
pnpm eslint --fix .
```

### 4.2 Naming

| Type | Convention | Example |
|------|------------|---------|
| Files | kebab-case | `document-card.tsx` |
| Components | PascalCase | `DocumentCard` |
| Hooks | camelCase with `use` prefix | `useDocument` |
| Types / interfaces | PascalCase | `Document` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_PAGE_SIZE` |
| Boolean vars | is/has/should prefix | `isLoading` |
| Event handlers | handle prefix | `handleSubmit` |

### 4.3 Imports

```tsx
// 1. React / external
import { useQuery } from "@tanstack/react-query"

// 2. internal absolute
import { Card } from "@/components/ui/card"
import { useDocument } from "@/entities/document"

// 3. relative within segment
import { DocumentHeader } from "./DocumentHeader"

// 4. types
import type { Document } from "@/entities/document"
```

### 4.4 Components

```tsx
interface DocumentCardProps {
  document: Document
  onClick?: (document: Document) => void
}

export function DocumentCard({ document, onClick }: DocumentCardProps) {
  return (
    <Card onClick={() => onClick?.(document)}>
      <span>{document.title}</span>
    </Card>
  )
}
```

### 4.5 Hooks

```tsx
export function useDocument(id: string) {
  return useQuery({
    queryKey: documentKeys.detail(id),
    queryFn: () => fetchDocument(id),
  })
}
```

### 4.6 Types

- Предпочитать `type` для объектов и union.
- `interface` только для public API / OOP shape.
- Никогда не использовать `any`. Использовать `unknown` + narrow.

### 4.7 Comments

```ts
// Bad
// increment i
i++

// Good
// Compensate for zero-based index when displaying row number.
const rowNumber = index + 1
```

### 4.8 Tests

```ts
import { render, screen } from "@testing-library/react"
import { DocumentCard } from "./DocumentCard"

describe("DocumentCard", () => {
  it("renders title", () => {
    render(<DocumentCard document={{ id: "1", title: "Release notes" } as Document} />)
    expect(screen.getByText("Release notes")).toBeInTheDocument()
  })
})
```

## 5. Commits

- Conventional commits.
- Формат: `type(scope): subject`.

| Type | Use |
|------|-----|
| `feat` | Новая фича |
| `fix` | Исправление бага |
| `docs` | Документация |
| `refactor` | Рефакторинг без изменения поведения |
| `test` | Тесты |
| `chore` | Сборка, deps, CI |
| `perf` | Производительность |

Примеры:

```
feat(documents): add revision publishing
fix(auth): refresh cookie path
refactor(evidence): split EvidenceFeed component
docs(api): clarify evidence endpoints
```

## 6. PR Rules

- PR должен быть небольшим (max 400-500 строк).
- Все CI checks green.
- Self-review перед запросом review.
- Reviewer назначается владельцем релевантной области.
- Merge только после approve.

## 7. File Organization

- Один публичный компонент/хук на файл.
- Стили — Tailwind utility, no CSS-in-JS.
- Константы — рядом с использованием или в `shared/config`.

## 8. Lint Configs

### 8.1 Rust

```toml
# clippy.toml
avoid-breaking-exported-api = false
doc-markdown = true
```

### 8.2 TypeScript

- `@typescript-eslint/recommended-type-checked`.
- `eslint-plugin-import` с alias `@/`.
- `eslint-plugin-react-hooks`.
- `eslint-plugin-boundaries` для FSD.

## 9. Documentation

- Публичные API методы Rust — doc comments.
- Сложные frontend функции — JSDoc.
- Любые non-obvious решения — запись в `docs/adr/`.

## 10. Prohibited

- `unwrap()` / `expect()` в production Rust (кроме startup).
- `any` в TypeScript.
- `console.log` в production (использовать logger).
- Inline styles (`style={{}}`).
- Magic numbers/строки без констант.
- Copy-pasted large blocks без выноса в функцию/компонент.

## 11. Git Workflow

- Main branch: `main`.
- Feature branches: `feat/wiki-123-short-desc`.
- Fix branches: `fix/short-desc`.
- Rebase before merge; no merge commits if possible.
- Force-push — только в своей feature-ветке.

## 12. API Versioning in Code

- Все REST endpoint под `/api/v1`.
- DTO именуются с версией только при необходимости: `DocumentResponseV1`.
- Deprecation — через заголовок `Deprecation` + docs.

## 13. Configuration

- Никаких secrets в коде.
- Все env vars с префиксом `WIKI_`.
- Валидация config при старте; fail fast.

## 14. Security

- SQL только через parameterized queries.
- Никакого `eval` / `innerHTML` с пользовательским контентом.
- CSP headers в production.
- Sanitize filenames и user input.

## 15. Performance

- Rust: avoid unnecessary clones, use `Arc` / references.
- React: use `React.memo` only after profiling.
- DB: always index query predicates.

## 16. Accessibility

- All interactive elements focusable.
- Semantic HTML.
- ARIA labels where text label отсутствует.
- Color contrast ≥ 4.5:1.
## References

- `docs/ARCHITECTURE.md`
- `docs/AGENTS.md`
- `docs/TESTING.md`
