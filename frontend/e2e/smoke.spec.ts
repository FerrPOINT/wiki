import { expect, test, type Page, type Route } from '@playwright/test'

const baseURL =
  process.env.PLAYWRIGHT_BASE_URL ??
  `http://localhost:${process.env.PLAYWRIGHT_PREVIEW_PORT ?? '4174'}`

const now = '2026-08-31T10:00:00Z'
const user = {
  id: '00000000-0000-0000-0000-000000000001',
  email: 'demo@example.com',
  username: 'demo',
  display_name: 'Демо пользователь',
  role: 'admin',
  is_system_admin: true,
  active: true,
}
const editorUser = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'editor@example.com',
  username: 'editor',
  display_name: 'Редактор',
  role: 'user',
  is_system_admin: false,
  active: true,
}
const spaceMember = {
  user_id: user.id,
  email: user.email,
  display_name: user.display_name,
  role: 'admin',
  joined_at: now,
}
const evidence = {
  id: 'evidence-smoke',
  space_key: 'SDLC',
  document_id: 'product-requirements',
  task_key: 'SDLC-42',
  phase_key: 'implementation',
  title: 'Материал smoke-проверки фронта',
  evidence_type: 'external_url',
  url: 'https://ci.local/jobs/wiki-smoke',
  attachment_id: null,
  checksum: null,
  created_by: user.id,
  created_at: now,
}
const document = {
  id: 'product-requirements',
  space_key: 'SDLC',
  parent_id: null,
  slug: 'product-requirements',
  title: 'Требования к Wiki MVP',
  document_type: 'requirements',
  status: 'published',
  body_markdown:
    '# Требования к Wiki MVP\n\nБазовый документ для пространств, документов, связей с задачами и фазами, материалов, поиска и аудита.',
  draft_markdown:
    '# Требования к Wiki MVP\n\nБазовый документ для пространств, документов, связей с задачами и фазами, материалов, поиска и аудита.',
  current_revision: {
    id: 'revision-product-requirements-1',
    document_id: 'product-requirements',
    version: 1,
    title: 'Требования к Wiki MVP',
    body_markdown:
      '# Требования к Wiki MVP\n\nБазовый документ для пространств, документов, связей с задачами и фазами, материалов, поиска и аудита.',
    summary: 'Исходные требования MVP',
    author_id: user.id,
    published_at: now,
  },
  task_keys: ['SDLC-42'],
  phase_keys: ['implementation'],
  evidence: [evidence],
  created_by: user.id,
  updated_by: user.id,
  created_at: now,
  updated_at: now,
}
const documentSummary = {
  id: document.id,
  slug: document.slug,
  title: document.title,
  document_type: document.document_type,
  status: document.status,
  updated_at: document.updated_at,
}
const task = {
  space_key: 'SDLC',
  task_key: 'SDLC-42',
  title: document.title,
  document_count: 1,
  evidence_count: 1,
  documents: [documentSummary],
  evidence: [evidence],
}
const phase = {
  space_key: 'SDLC',
  phase_key: 'implementation',
  title: 'implementation',
  document_count: 1,
  evidence_count: 1,
  documents: [documentSummary],
  evidence: [evidence],
}
const settings = {
  instance_name: 'Wiki',
  api_base_path: '/api/v1',
  default_space_key: 'SDLC',
  default_language: 'ru',
  timezone: 'Europe/Moscow',
  registration_enabled: true,
  public_links_enabled: false,
  search_backend: 'PostgreSQL FTS',
  storage_backend: 'local',
  max_upload_bytes: 26214400,
  markdown_renderer: 'comrak',
  html_sanitizer: 'ammonia',
}
const template = {
  id: 'requirements',
  name: 'Требования',
  document_type: 'requirements',
  body_markdown: '# Требования\n\n## Контекст\n\n## Решения\n\n## Проверки\n',
}

function routeJson(route: Route, body: unknown, status = 200) {
  return route.fulfill({
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  })
}

async function installWikiApiMocks(page: Page) {
  let currentDocument = { ...document }
  let currentRevisions = [document.current_revision]
  let currentTemplates = [template]
  let currentUsers = [user, editorUser]
  const documentDraftRequests: Array<{ title?: string; content_markdown: string }> = []
  const documentPublishRequests: Array<{ summary?: string | null }> = []
  const templateCreateRequests: Array<{
    name: string
    document_type: string
    body_markdown: string
  }> = []
  const userUpdateRequests: Array<{ userId: string; role?: string; active?: boolean }> = []
  const searchRequests: string[] = []
  const evidenceRequests: string[] = []

  await page.route('**/api/v1/**', (route) => {
    const request = route.request()
    const url = new URL(request.url())
    const path = url.pathname.replace('/api/v1', '')
    const method = request.method()

    if (method === 'POST' && path === '/auth/login') {
      return routeJson(route, {
        access_token: 'demo-token',
        refresh_token: 'demo-refresh',
        token_type: 'Bearer',
        user_id: user.id,
        email: user.email,
        username: user.username,
        display_name: user.display_name,
      })
    }
    if (method === 'POST' && path === '/auth/refresh') {
      return routeJson(route, { access_token: 'demo-token', refresh_token: 'demo-refresh' })
    }
    if (method === 'POST' && path === '/auth/logout') return route.fulfill({ status: 204 })
    if (method === 'GET' && path === '/users/me') return routeJson(route, user)
    if (method === 'GET' && path === '/users') return routeJson(route, { users: currentUsers })
    if (method === 'PUT' && path.startsWith('/users/')) {
      const userId = decodeURIComponent(path.split('/').pop() ?? '')
      const body = request.postDataJSON() as { role?: string; active?: boolean }
      userUpdateRequests.push({ userId, role: body.role, active: body.active })
      currentUsers = currentUsers.map((item) =>
        item.id === userId
          ? {
              ...item,
              role: body.role ?? item.role,
              is_system_admin: body.role === 'admin',
              active: body.active ?? item.active,
            }
          : item,
      )
      return routeJson(
        route,
        currentUsers.find((item) => item.id === userId),
      )
    }
    if (method === 'GET' && path === '/settings') return routeJson(route, settings)
    if (method === 'GET' && path === '/spaces') {
      return routeJson(route, {
        spaces: [
          {
            id: 'space-sdlc',
            key: 'SDLC',
            name: 'База знаний SDLC',
            description: 'Основное пространство Wiki для документов SDLC',
            owner_id: user.id,
            status: 'active',
            document_count: 1,
            member_count: 1,
            created_at: now,
            updated_at: now,
          },
        ],
      })
    }
    if (method === 'GET' && path === '/spaces/SDLC/members') {
      return routeJson(route, { members: [spaceMember] })
    }
    if (method === 'GET' && path === '/spaces/SDLC/tree') {
      return routeJson(route, {
        space_key: 'SDLC',
        documents: [
          {
            id: document.id,
            slug: document.slug,
            title: document.title,
            document_type: document.document_type,
            status: document.status,
            children: [],
          },
        ],
      })
    }
    if (method === 'GET' && path === '/documents/product-requirements') {
      return routeJson(route, currentDocument)
    }
    if (method === 'GET' && path === '/documents/product-requirements/revisions') {
      return routeJson(route, { revisions: currentRevisions })
    }
    if (method === 'GET' && path.startsWith('/documents/product-requirements/revisions/')) {
      const revisionId = path.split('/').pop()
      const revision = currentRevisions.find((item) => item.id === revisionId)
      return revision
        ? routeJson(route, revision)
        : routeJson(route, { code: 'NOT_FOUND', message: 'Revision not found' }, 404)
    }
    if (method === 'PUT' && path === '/documents/product-requirements/draft') {
      const body = request.postDataJSON() as { title?: string; content_markdown: string }
      documentDraftRequests.push(body)
      currentDocument = {
        ...currentDocument,
        title: body.title ?? currentDocument.title,
        status: 'draft',
        draft_markdown: body.content_markdown,
        updated_at: now,
      }
      return routeJson(route, currentDocument)
    }
    if (method === 'POST' && path === '/documents/product-requirements/publish') {
      const body = request.postDataJSON() as { summary?: string | null }
      documentPublishRequests.push(body)
      const revision = {
        ...document.current_revision,
        id: 'revision-product-requirements-2',
        version: 2,
        title: currentDocument.title,
        body_markdown: currentDocument.draft_markdown,
        summary: body.summary ?? null,
        published_at: now,
      }
      currentDocument = {
        ...currentDocument,
        status: 'published',
        body_markdown: currentDocument.draft_markdown,
        current_revision: revision,
        updated_at: now,
      }
      currentRevisions = [revision, ...currentRevisions]
      return routeJson(route, revision)
    }
    if (method === 'POST' && path === '/documents/product-requirements/archive') {
      currentDocument = { ...currentDocument, status: 'archived', updated_at: now }
      return routeJson(route, currentDocument)
    }
    if (method === 'POST' && path === '/documents/product-requirements/move') {
      const body = request.postDataJSON() as { parent_id?: string | null }
      currentDocument = { ...currentDocument, parent_id: body.parent_id ?? null, updated_at: now }
      return routeJson(route, currentDocument)
    }
    if (method === 'GET' && path === '/spaces/SDLC/tasks')
      return routeJson(route, { tasks: [task] })
    if (method === 'GET' && path === '/spaces/SDLC/tasks/SDLC-42') return routeJson(route, task)
    if (method === 'GET' && path === '/spaces/SDLC/phases') {
      return routeJson(route, { phases: [phase] })
    }
    if (method === 'GET' && path === '/spaces/SDLC/phases/implementation') {
      return routeJson(route, phase)
    }
    if (method === 'GET' && path === '/evidence') {
      evidenceRequests.push(url.search)
      return routeJson(route, { evidence: [evidence] })
    }
    if (method === 'GET' && path === '/templates') {
      return routeJson(route, {
        templates: currentTemplates,
      })
    }
    if (method === 'POST' && path === '/templates') {
      const body = request.postDataJSON() as {
        name: string
        document_type: string
        body_markdown: string
      }
      templateCreateRequests.push(body)
      const created = {
        id: body.name.toLowerCase().replace(/\s+/g, '-'),
        name: body.name,
        document_type: body.document_type,
        body_markdown: body.body_markdown,
      }
      currentTemplates = [created, ...currentTemplates]
      return routeJson(route, created, 201)
    }
    if (method === 'GET' && path === '/audit-log') {
      return routeJson(route, {
        entries: [
          {
            id: 'audit-initial',
            actor_id: user.id,
            action: 'wiki.seeded',
            entity_type: 'space',
            entity_id: 'SDLC',
            created_at: now,
          },
        ],
      })
    }
    if (method === 'GET' && path === '/search') {
      searchRequests.push(url.search)
      return routeJson(route, {
        results: [
          {
            id: document.id,
            result_type: 'document',
            title: document.title,
            space_key: document.space_key,
            url: `/documents/${document.slug}`,
            snippet: document.body_markdown,
            updated_at: document.updated_at,
          },
          {
            id: evidence.id,
            result_type: 'evidence',
            title: evidence.title,
            space_key: evidence.space_key,
            url: `/evidence?id=${evidence.id}`,
            snippet: evidence.url,
            updated_at: evidence.created_at,
          },
        ],
      })
    }

    return routeJson(route, { error: `Unhandled mock route ${method} ${path}` }, 404)
  })

  return {
    documentDraftRequests,
    documentPublishRequests,
    evidenceRequests,
    searchRequests,
    templateCreateRequests,
    userUpdateRequests,
  }
}

test.describe('wiki smoke', () => {
  test('login and navigate through wiki shell pages', async ({ page }) => {
    const apiMocks = await installWikiApiMocks(page)
    await page.goto(`${baseURL}/login`)
    await page.getByRole('textbox').nth(0).fill('demo@example.com')
    await page.getByRole('textbox').nth(1).fill('demo')
    await page.getByRole('button', { name: /войти/i }).click()

    await expect(page).toHaveURL(`${baseURL}/`, { timeout: 10_000 })
    await expect(page.getByRole('heading', { name: 'Wiki', exact: true })).toBeVisible()

    await page.goto(`${baseURL}/spaces`)
    await expect(page.getByRole('heading', { name: 'Пространства' })).toBeVisible()
    await expect(page.getByText('База знаний SDLC')).toBeVisible()

    await page.goto(`${baseURL}/documents/new`)
    await expect(page.getByRole('heading', { name: 'Новый документ' })).toBeVisible()

    await page.goto(`${baseURL}/documents/product-requirements`)
    await expect(page.getByRole('heading', { name: 'Требования к Wiki MVP' })).toBeVisible()
    await page.getByLabel('Markdown черновика').fill('# Обновлено\n\nЧерновик из e2e.')
    await page.getByRole('button', { name: 'Сохранить', exact: true }).click()
    await expect.poll(() => apiMocks.documentDraftRequests.length).toBe(1)
    await expect(page.getByText('Черновик сохранён')).toBeVisible()
    await page.getByLabel('Комментарий к публикации').fill('E2E publish')
    await page.getByRole('button', { name: 'Опубликовать', exact: true }).click()
    await expect
      .poll(() =>
        apiMocks.documentPublishRequests.some((request) => request.summary === 'E2E publish'),
      )
      .toBe(true)
    await expect(page.getByText('Опубликована ревизия 2')).toBeVisible()
    await page.getByRole('button', { name: 'Открыть' }).first().click()
    await expect(page.getByRole('heading', { name: 'Снимок ревизии' })).toBeVisible()
    await expect(page.getByText('Ревизия 2: Требования к Wiki MVP')).toBeVisible()

    await page.goto(`${baseURL}/tasks/SDLC-42`)
    await expect(page.getByRole('heading', { name: 'SDLC-42' })).toBeVisible()

    await page.goto(`${baseURL}/phases/implementation`)
    await expect(page.getByRole('heading', { name: 'implementation' })).toBeVisible()

    await page.goto(`${baseURL}/evidence`)
    await expect(page.getByRole('heading', { name: 'Материалы' })).toBeVisible()
    await page.getByLabel('Фильтр документа').fill('product-requirements')
    await expect
      .poll(() =>
        apiMocks.evidenceRequests.some((query) =>
          query.includes('document_id=product-requirements'),
        ),
      )
      .toBe(true)

    await page.goto(`${baseURL}/search`)
    await expect(page.getByRole('heading', { name: 'Поиск' })).toBeVisible()
    await page.getByRole('button', { name: 'Требования', exact: true }).click()
    await expect
      .poll(() =>
        apiMocks.searchRequests.some((query) => query.includes('document_type=requirements')),
      )
      .toBe(true)

    await page.goto(`${baseURL}/templates`)
    await expect(page.getByRole('heading', { name: 'Шаблоны' })).toBeVisible()
    await page.getByLabel('Название шаблона').fill('Шаблон релиза')
    await page.getByLabel('Тип документа').selectOption('release_note')
    await page.getByLabel('Markdown шаблона').fill('# Релиз\n\n## Проверки\n')
    await page.getByRole('button', { name: 'Создать шаблон' }).click()
    await expect
      .poll(() =>
        apiMocks.templateCreateRequests.some((request) => request.name === 'Шаблон релиза'),
      )
      .toBe(true)
    await expect(page.getByText('Шаблон релиза')).toBeVisible()

    await page.goto(`${baseURL}/users`)
    await expect(page.getByRole('heading', { name: 'Пользователи', exact: true })).toBeVisible()
    const editorRole = page.getByLabel('Роль пользователя editor@example.com')
    await editorRole.selectOption('admin')
    await page.getByLabel('Статус пользователя editor@example.com').selectOption('disabled')
    await editorRole
      .locator('xpath=ancestor::form')
      .getByRole('button', { name: 'Сохранить' })
      .click()
    await expect
      .poll(() =>
        apiMocks.userUpdateRequests.some(
          (request) =>
            request.userId === editorUser.id &&
            request.role === 'admin' &&
            request.active === false,
        ),
      )
      .toBe(true)

    await page.goto(`${baseURL}/settings`)
    await expect(page.getByRole('heading', { name: 'Настройки' })).toBeVisible()
    await expect(page.locator('input[value="PostgreSQL FTS"]')).toBeVisible()

    await page.goto(`${baseURL}/admin`)
    await expect(page.getByRole('heading', { name: 'Администрирование' })).toBeVisible()
    await expect(page.getByRole('heading', { name: 'Состояние инстанса' })).toBeVisible()
    await expect(page.getByText('Файлы до 25 МБ')).toBeVisible()
  })
})
