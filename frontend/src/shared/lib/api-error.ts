type ApiErrorDetailLike = {
  field?: unknown
  message?: unknown
}

type ApiErrorLike = {
  code?: unknown
  details?: unknown
  message?: unknown
}

const codeMessages: Record<string, string> = {
  CONFLICT: 'Конфликт данных',
  FORBIDDEN: 'Недостаточно прав для действия',
  NOT_FOUND: 'Объект не найден',
  UNAUTHORIZED: 'Нужно войти заново',
  VALIDATION_ERROR: 'Проверьте заполнение полей',
}

const fieldLabels: Record<string, string> = {
  attachment_id: 'Файл',
  content_markdown: 'Markdown',
  document_id: 'Документ',
  document_type: 'Тип документа',
  email: 'Email',
  password: 'Пароль',
  phase_key: 'Фаза',
  role: 'Роль',
  slug: 'Адрес',
  space: 'Пространство',
  space_key: 'Пространство',
  task_key: 'Задача',
  title: 'Название',
  url: 'URL',
  username: 'Логин',
}

export function formatApiErrorForUser(error: unknown, fallback: string): string {
  const apiError = readApiErrorLike(error)
  if (!apiError) return fallback

  const code = readString(apiError.code)
  const rawMessage = stripTechnicalSuffixes(readString(apiError.message))
  const base = codeMessages[code] || rawMessage || fallback
  const details = formatDetails(apiError.details)

  return details ? `${base}: ${details}` : base
}

export function formatFirstApiErrorForUser(errors: unknown[], fallback: string): string {
  const error = errors.find((item) => Boolean(item))
  return error ? formatApiErrorForUser(error, fallback) : ''
}

function readApiErrorLike(error: unknown): ApiErrorLike | null {
  if (!error || typeof error !== 'object') return null
  return error as ApiErrorLike
}

function readString(value: unknown): string {
  return typeof value === 'string' ? value.trim() : ''
}

function stripTechnicalSuffixes(message: string): string {
  return message
    .split(';')
    .map((part) => part.trim())
    .filter((part) => part && !part.startsWith('requestId=') && !part.startsWith('details='))
    .join('; ')
}

function formatDetails(details: unknown): string {
  if (!Array.isArray(details)) return ''

  return details
    .map((detail) => formatDetail(detail as ApiErrorDetailLike))
    .filter((detail): detail is string => Boolean(detail))
    .join(', ')
}

function formatDetail(detail: ApiErrorDetailLike): string {
  if (!detail || typeof detail !== 'object') return ''

  const field = readString(detail.field)
  const message = readString(detail.message)
  const label = field ? (fieldLabels[field] ?? field) : ''

  if (label && message) return `${label}: ${message}`
  return label || message
}
