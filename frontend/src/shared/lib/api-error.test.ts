import { describe, expect, it } from 'vitest'

import { formatApiErrorForUser, formatFirstApiErrorForUser } from './api-error'

describe('formatApiErrorForUser', () => {
  it('renders permission failures as a Russian user-facing message', () => {
    expect(formatApiErrorForUser({ code: 'FORBIDDEN', message: 'Forbidden' }, 'fallback')).toBe(
      'Недостаточно прав для действия',
    )
  })

  it('keeps validation details but hides request identifiers', () => {
    const message = formatApiErrorForUser(
      {
        code: 'VALIDATION_ERROR',
        message: 'Validation failed; details=title: required; requestId=req-123',
        details: [
          { field: 'title', message: 'required' },
          { field: 'content_markdown', message: 'too short' },
        ],
      },
      'fallback',
    )

    expect(message).toBe('Проверьте заполнение полей: Название: required, Markdown: too short')
    expect(message).not.toContain('requestId')
    expect(message).not.toContain('details=')
  })

  it('falls back to a readable plain error message for unknown errors', () => {
    expect(formatApiErrorForUser(new Error('Сервис временно недоступен'), 'fallback')).toBe(
      'Сервис временно недоступен',
    )
  })

  it('uses the fallback when the API error has no useful message', () => {
    expect(formatApiErrorForUser({ code: 'UNKNOWN', message: '' }, 'Не удалось')).toBe('Не удалось')
  })

  it('formats the first present error from aggregate page queries', () => {
    expect(
      formatFirstApiErrorForUser([null, { code: 'UNAUTHORIZED', message: 'Unauthorized' }], 'x'),
    ).toBe('Нужно войти заново')
  })

  it('returns an empty message when aggregate page queries have no error', () => {
    expect(formatFirstApiErrorForUser([null, undefined, false], 'fallback')).toBe('')
  })
})
