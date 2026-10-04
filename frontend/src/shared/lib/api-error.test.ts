import { describe, expect, it } from 'vitest'

import { formatApiErrorForUser, formatFirstApiErrorForUser, hasApiErrorCode } from './api-error'

describe('formatApiErrorForUser', () => {
  it.each([
    ['INTERNAL_ERROR', 'Ошибка сервера. Попробуйте ещё раз позже'],
    ['UNAVAILABLE', 'Сервис временно недоступен. Попробуйте ещё раз позже'],
  ])('renders %s safely without internal diagnostics', (code, expected) => {
    expect(
      formatApiErrorForUser(
        {
          code,
          message: 'internal server error; requestId=private-id',
          details: [{ field: 'database', message: 'private diagnostic' }],
        },
        '',
      ),
    ).toBe(expected)
  })

  it.each(['INTERNAL_ERROR', 'UNAVAILABLE'])('keeps the page-specific message for %s', (code) => {
    expect(
      formatApiErrorForUser(
        { code, message: 'private diagnostic' },
        'Не удалось загрузить пространства',
      ),
    ).toBe('Не удалось загрузить пространства')
  })

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

  it('identifies a specific API error code without trusting arbitrary values', () => {
    expect(hasApiErrorCode({ code: 'CONFLICT' }, 'CONFLICT')).toBe(true)
    expect(hasApiErrorCode({ code: 'FORBIDDEN' }, 'CONFLICT')).toBe(false)
    expect(hasApiErrorCode(new Error('CONFLICT'), 'CONFLICT')).toBe(false)
  })
})
