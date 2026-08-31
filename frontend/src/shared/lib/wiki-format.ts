const documentStatuses: Record<string, string> = {
  draft: 'черновик',
  published: 'опубликован',
  archived: 'архив',
}

const documentTypes: Record<string, string> = {
  page: 'страница',
  requirements: 'требования',
  research_note: 'исследование',
  'research-note': 'исследование',
  implementation_note: 'реализация',
  'implementation-note': 'реализация',
  test_plan: 'план проверки',
  'test-plan': 'план проверки',
  release_note: 'релиз',
  'release-note': 'релиз',
}

const evidenceTypes: Record<string, string> = {
  external_url: 'ссылка',
  uploaded_file: 'файл',
}

export function formatDocumentStatus(status: string | undefined): string {
  if (!status) return 'неизвестно'
  return documentStatuses[status] ?? status
}

export function formatDocumentType(type: string | undefined): string {
  if (!type) return 'страница'
  return documentTypes[type] ?? type
}

export function formatEvidenceType(type: string | undefined): string {
  if (!type) return 'материал'
  return evidenceTypes[type] ?? type
}

export function formatDateTime(value: string | undefined): string {
  if (!value) return 'нет даты'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return new Intl.DateTimeFormat('ru-RU', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

export function shortText(value: string | undefined | null, fallback = 'Без описания'): string {
  const text = value?.trim()
  return text && text.length > 0 ? text : fallback
}

export function formatBytes(value: number | undefined): string {
  if (value === undefined || !Number.isFinite(value)) return 'не задано'
  const units = ['Б', 'КБ', 'МБ', 'ГБ']
  let size = value
  let unit = 0
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024
    unit += 1
  }
  const precision = unit === 0 || size >= 10 ? 0 : 1
  return `${size.toFixed(precision)} ${units[unit]}`
}
