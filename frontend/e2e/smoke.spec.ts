import { expect, test, type Page, type Route } from '@playwright/test'
import { exportJWK, generateKeyPair, SignJWT } from 'jose'

const baseURL =
  process.env.PLAYWRIGHT_BASE_URL ??
  `http://localhost:${process.env.PLAYWRIGHT_PREVIEW_PORT ?? '4174'}`

const now = '2026-08-31T10:00:00Z'
const user = {
  id: '00000000-0000-0000-0000-000000000001',
  email: 'admin@example.com',
  username: 'admin',
  display_name: 'Администратор',
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
  space_key: 'BASE',
  document_id: 'product-requirements',
  task_key: 'BASE-42',
  phase_key: 'implementation',
  title: 'Материал smoke-проверки фронта',
  evidence_type: 'external_url',
  url: 'https://ci.local/jobs/wiki-smoke',
  attachment_id: null,
  checksum: null,
  created_by: user.id,
  created_at: now,
}
const fileAttachment = {
  id: 'attachment-build-log',
  checksum: 'sha256:fileabc123',
  content_type: 'text/plain',
  file_name: 'build.log',
  size_bytes: 2048,
  uploaded_at: now,
  uploaded_by: user.id,
}
const fileEvidence = {
  id: 'evidence-file-smoke',
  space_key: 'BASE',
  document_id: 'product-requirements',
  task_key: 'BASE-42',
  phase_key: 'testing',
  title: 'Лог сборки',
  evidence_type: 'uploaded_file',
  url: null,
  attachment_id: fileAttachment.id,
  checksum: fileAttachment.checksum,
  created_by: user.id,
  created_at: now,
}
const documentBodyMarkdown =
  '# Требования к Wiki MVP\n\nБазовый документ для пространств, документов, связей с задачами и фазами, материалов, поиска и аудита.'

function escapeHtml(value: string) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
}

function renderMockMarkdown(markdown: string) {
  return markdown
    .split(/\n{2,}/)
    .map((block) => block.trim())
    .filter(Boolean)
    .map((block) => {
      if (block.startsWith('# ')) return `<h1>${escapeHtml(block.slice(2))}</h1>`
      if (block.startsWith('## ')) return `<h2>${escapeHtml(block.slice(3))}</h2>`
      return `<p>${escapeHtml(block).replaceAll('\n', '<br />')}</p>`
    })
    .join('\n')
}
const document = {
  id: 'product-requirements',
  space_key: 'BASE',
  parent_id: null,
  slug: 'product-requirements',
  title: 'Требования к Wiki MVP',
  document_type: 'requirements',
  status: 'published',
  can_edit: true,
  body_markdown: documentBodyMarkdown,
  body_html: renderMockMarkdown(documentBodyMarkdown),
  draft_markdown: documentBodyMarkdown,
  current_revision: {
    id: 'revision-product-requirements-1',
    document_id: 'product-requirements',
    version: 1,
    title: 'Требования к Wiki MVP',
    body_markdown: documentBodyMarkdown,
    body_html: renderMockMarkdown(documentBodyMarkdown),
    summary: 'Исходные требования MVP',
    author_id: user.id,
    published_at: now,
  },
  task_keys: ['BASE-42'],
  phase_keys: ['implementation'],
  evidence: [evidence, fileEvidence],
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
  space_key: 'BASE',
  task_key: 'BASE-42',
  title: document.title,
  document_count: 1,
  evidence_count: 2,
  documents: [documentSummary],
  evidence: [evidence, fileEvidence],
}
const phase = {
  space_key: 'BASE',
  phase_key: 'implementation',
  title: 'implementation',
  document_count: 1,
  evidence_count: 2,
  documents: [documentSummary],
  evidence: [evidence, fileEvidence],
}
const settings = {
  instance_name: 'Wiki',
  api_base_path: '/api/v1',
  default_space_key: 'BASE',
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
  const { privateKey, publicKey } = await generateKeyPair('ES256')
  const jwk = await exportJWK(publicKey)
  const oidcValue = Buffer.alloc(32, 7).toString('base64url')
  const issuer = 'http://localhost:7701'

  // Make browser-only PKCE values predictable so the mocked callback can
  // validate the same state and nonce without reaching Central Auth.
  await page.addInitScript(() => {
    const original = crypto.getRandomValues.bind(crypto)
    crypto.getRandomValues = ((array: Uint8Array) => {
      if (array.byteLength === 32) array.fill(7)
      else original(array)
      return array
    }) as typeof crypto.getRandomValues
  })

  await page.route(`${issuer}/oidc/**`, async (route: Route) => {
    const request = route.request()
    const url = new URL(request.url())
    if (url.pathname === '/oidc/authorize') {
      const callbackUrl = `${baseURL}/sso/callback?code=mock-code&state=${url.searchParams.get('state')}`
      return route.fulfill({
        status: 200,
        contentType: 'text/html',
        body: `<!doctype html><script>location.replace(${JSON.stringify(callbackUrl)})</script>`,
      })
    }
    if (url.pathname === '/oidc/jwks') return routeJson(route, { keys: [jwk] })
    if (url.pathname === '/oidc/token') {
      const idToken = await new SignJWT({
        email: user.email,
        name: user.display_name,
        nonce: oidcValue,
      })
        .setProtectedHeader({ alg: 'ES256' })
        .setIssuer(issuer)
        .setAudience('wiki')
        .setSubject(user.id)
        .setIssuedAt()
        .setExpirationTime('5m')
        .sign(privateKey)
      return routeJson(route, { access_token: 'demo-token', id_token: idToken, expires_in: 300 })
    }
    return route.fulfill({ status: 404 })
  })

  let currentDocument = { ...document }
  let currentRevisions = [document.current_revision]
  let currentTemplates = [template]
  let currentTask = { ...task, documents: [...task.documents], evidence: [...task.evidence] }
  let currentPhase = { ...phase, documents: [...phase.documents], evidence: [...phase.evidence] }
  let currentUsers = [user, editorUser]
  const documentDraftRequests: Array<{ title?: string; content_markdown: string }> = []
  const documentPublishRequests: Array<{ summary?: string | null }> = []
  const taskDocumentLinkRequests: Array<{ document_id: string }> = []
  const phaseDocumentLinkRequests: Array<{ document_id: string }> = []
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
            key: 'BASE',
            name: 'База знаний Base',
            description: 'Основное пространство Wiki для документов платформы Base',
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
    if (method === 'POST' && path === '/spaces/BASE/archive') {
      return routeJson(route, {
        id: 'space-sdlc',
        key: 'BASE',
        name: 'База знаний Base',
        description: 'Основное пространство Wiki для документов платформы Base',
        owner_id: user.id,
        status: 'archived',
        document_count: 1,
        member_count: 1,
        created_at: now,
        updated_at: now,
      })
    }
    if (method === 'GET' && path === '/spaces/BASE/members') {
      return routeJson(route, { members: [spaceMember] })
    }
    if (method === 'GET' && path === '/spaces/BASE/tree') {
      return routeJson(route, {
        space_key: 'BASE',
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
        body_html: renderMockMarkdown(currentDocument.draft_markdown),
        summary: body.summary ?? null,
        published_at: now,
      }
      currentDocument = {
        ...currentDocument,
        status: 'published',
        body_markdown: currentDocument.draft_markdown,
        body_html: renderMockMarkdown(currentDocument.draft_markdown),
        current_revision: revision,
        updated_at: now,
      }
      currentRevisions = [revision, ...currentRevisions]
      return routeJson(route, revision)
    }
    if (method === 'POST' && path === '/documents/product-requirements/archive') {
      currentDocument = { ...currentDocument, status: 'archived', can_edit: false, updated_at: now }
      return routeJson(route, currentDocument)
    }
    if (method === 'POST' && path === '/documents/product-requirements/move') {
      const body = request.postDataJSON() as { parent_id?: string | null }
      currentDocument = { ...currentDocument, parent_id: body.parent_id ?? null, updated_at: now }
      return routeJson(route, currentDocument)
    }
    if (method === 'GET' && path === '/spaces/BASE/tasks')
      return routeJson(route, { tasks: [currentTask] })
    if (method === 'GET' && path === '/spaces/BASE/tasks/BASE-42')
      return routeJson(route, currentTask)
    if (method === 'POST' && path === '/spaces/BASE/tasks/BASE-42/links/documents') {
      const body = request.postDataJSON() as { document_id: string }
      taskDocumentLinkRequests.push(body)
      currentTask = {
        ...currentTask,
        documents: currentTask.documents.some((item) => item.id === body.document_id)
          ? currentTask.documents
          : [documentSummary, ...currentTask.documents],
        document_count: currentTask.documents.some((item) => item.id === body.document_id)
          ? currentTask.document_count
          : currentTask.document_count + 1,
      }
      return routeJson(route, currentTask)
    }
    if (method === 'GET' && path === '/spaces/BASE/phases') {
      return routeJson(route, { phases: [currentPhase] })
    }
    if (method === 'GET' && path === '/spaces/BASE/phases/implementation') {
      return routeJson(route, currentPhase)
    }
    if (method === 'POST' && path === '/spaces/BASE/phases/implementation/links/documents') {
      const body = request.postDataJSON() as { document_id: string }
      phaseDocumentLinkRequests.push(body)
      currentPhase = {
        ...currentPhase,
        documents: currentPhase.documents.some((item) => item.id === body.document_id)
          ? currentPhase.documents
          : [documentSummary, ...currentPhase.documents],
        document_count: currentPhase.documents.some((item) => item.id === body.document_id)
          ? currentPhase.document_count
          : currentPhase.document_count + 1,
      }
      return routeJson(route, currentPhase)
    }
    if (method === 'GET' && path === '/evidence') {
      evidenceRequests.push(url.search)
      return routeJson(route, { evidence: [evidence, fileEvidence] })
    }
    if (method === 'GET' && path.startsWith('/evidence/')) {
      const evidenceId = decodeURIComponent(path.split('/').pop() ?? '')
      const item = [evidence, fileEvidence].find((entry) => entry.id === evidenceId)
      return item
        ? routeJson(route, item)
        : routeJson(route, { code: 'NOT_FOUND', message: 'Evidence not found' }, 404)
    }
    if (method === 'GET' && path === `/attachments/${fileAttachment.id}`) {
      return routeJson(route, fileAttachment)
    }
    if (method === 'GET' && path === `/attachments/${fileAttachment.id}/download`) {
      return route.fulfill({
        status: 200,
        contentType: fileAttachment.content_type,
        headers: {
          'Content-Disposition': `attachment; filename="${fileAttachment.file_name}"`,
        },
        body: 'downloaded bytes',
      })
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
            entity_id: 'BASE',
            request_id: 'mock-request',
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
    phaseDocumentLinkRequests,
    searchRequests,
    taskDocumentLinkRequests,
    templateCreateRequests,
    userUpdateRequests,
  }
}

async function gotoWiki(page: Page, path = '/') {
  const target = new URL(path, `${baseURL}/`).toString()
  await page.goto(target)
  await expect(page).toHaveURL(target, { timeout: 10_000 })
}

test.describe('wiki smoke', () => {
  test('keeps the platform shell active-route and keyboard drawer contract', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 })
    await installWikiApiMocks(page)
    await gotoWiki(page)
    await gotoWiki(page, '/tasks/BASE-42')
    await expect(page.getByRole('heading', { name: 'BASE-42' })).toBeVisible()

    const trigger = page.getByRole('button', { name: 'Открыть навигацию' })
    await trigger.click()
    const dialog = page.getByRole('dialog', { name: 'Навигация Wiki' })

    await expect(dialog).toBeVisible()
    await expect(dialog.getByRole('link', { name: 'Задачи' })).toHaveAttribute(
      'aria-current',
      'page',
    )
    await expect
      .poll(() => dialog.evaluate((element) => element.contains(document.activeElement)))
      .toBe(true)
    await page.keyboard.press('Shift+Tab')
    await expect
      .poll(() => dialog.evaluate((element) => element.contains(document.activeElement)))
      .toBe(true)
    await page.keyboard.press('Escape')
    await expect(dialog).toBeHidden()
    await expect(trigger).toBeFocused()
  })

  test('archives a space only after explicit confirmation and keeps its archived status visible', async ({
    page,
  }) => {
    await installWikiApiMocks(page)
    await gotoWiki(page)
    await gotoWiki(page, '/spaces')
    await page.getByRole('button', { name: /База знаний Base/ }).click()
    await page.getByRole('button', { name: 'Архивировать' }).click()

    const dialog = page.getByRole('alertdialog')
    const cancel = dialog.getByRole('button', { name: 'Отмена' })
    await expect(dialog).toContainText('Восстановление из интерфейса пока недоступно')
    await cancel.click()
    await expect(dialog).not.toBeVisible()

    await page.getByRole('button', { name: 'Архивировать' }).click()
    await dialog.getByRole('button', { name: 'Подтвердить' }).click()

    await expect(page.getByRole('status')).toContainText(
      'архивировано и остаётся доступным для чтения',
    )
    await expect(page.getByRole('button', { name: /База знаний Base/ })).toContainText(
      'архивировано',
    )
    await expect(page.getByRole('button', { name: 'Архивировать' })).not.toBeVisible()
    await page.screenshot({ path: 'test-results/wiki-space-archive.png', fullPage: true })
  })

  test('signs in through OIDC and navigates through wiki shell pages', async ({ page }) => {
    const apiMocks = await installWikiApiMocks(page)
    await gotoWiki(page)

    await expect(page).toHaveURL(`${baseURL}/`, { timeout: 10_000 })
    await expect(page.getByRole('heading', { name: 'Wiki', exact: true })).toBeVisible()

    await gotoWiki(page, '/spaces')
    await expect(page.getByRole('heading', { name: 'Пространства' })).toBeVisible()
    await expect(page.getByText('База знаний Base')).toBeVisible()

    await gotoWiki(page, '/documents/new')
    await expect(page.getByRole('heading', { name: 'Новый документ' })).toBeVisible()

    await gotoWiki(page, '/documents/product-requirements')
    await expect(
      page.locator('article > section').first().getByRole('heading', {
        name: 'Требования к Wiki MVP',
        exact: true,
      }),
    ).toBeVisible()
    await expect(
      page.locator('.wiki-rendered').first().getByText('Базовый документ для пространств'),
    ).toBeVisible()
    await page.getByRole('button', { name: 'Правка' }).click()
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
    const revisionTwo = page.getByRole('group', { name: 'Ревизия 2' })
    await expect(revisionTwo).toBeVisible()
    const openRevision = revisionTwo.getByRole('button', { name: 'Открыть ревизию 2' })
    await openRevision.click()
    const revisionDialog = page.getByRole('dialog', { name: 'Снимок ревизии' })
    await expect(revisionDialog).toBeVisible()
    await expect(revisionDialog.getByText('Ревизия 2: Требования к Wiki MVP')).toBeVisible()
    await revisionDialog.getByRole('button', { name: 'Закрыть' }).click()
    await expect(revisionDialog).toBeHidden()
    await expect(openRevision).toBeFocused()

    await gotoWiki(page, '/tasks/BASE-42')
    await expect(page.getByRole('heading', { name: 'BASE-42' })).toBeVisible()
    await page.getByLabel('Документ для задачи').fill('product-requirements')
    await page.getByRole('button', { name: 'Привязать' }).click()
    await expect
      .poll(() =>
        apiMocks.taskDocumentLinkRequests.some(
          (request) => request.document_id === 'product-requirements',
        ),
      )
      .toBe(true)
    await expect(page.getByText('Документ привязан к задаче')).toBeVisible()

    await gotoWiki(page, '/phases/implementation')
    await expect(page.getByRole('heading', { name: 'implementation' })).toBeVisible()
    await page.getByLabel('Документ для фазы').fill('product-requirements')
    await page.getByRole('button', { name: 'Привязать' }).click()
    await expect
      .poll(() =>
        apiMocks.phaseDocumentLinkRequests.some(
          (request) => request.document_id === 'product-requirements',
        ),
      )
      .toBe(true)
    await expect(page.getByText('Документ привязан к фазе')).toBeVisible()

    await gotoWiki(page, '/evidence')
    await expect(page.getByRole('heading', { name: 'Материалы' })).toBeVisible()
    await page.getByRole('button', { name: 'Открыть материал Лог сборки' }).click()
    await expect(page.getByRole('heading', { name: 'Выбранный материал' })).toBeVisible()
    await expect(page.getByText(fileAttachment.checksum)).toBeVisible()
    await expect(page.getByText(fileAttachment.file_name)).toBeVisible()
    await page.getByLabel('Фильтр документа').fill('product-requirements')
    await page.getByRole('button', { name: 'Найти' }).click()
    await expect
      .poll(() =>
        apiMocks.evidenceRequests.some((query) =>
          query.includes('document_id=product-requirements'),
        ),
      )
      .toBe(true)

    await gotoWiki(page, '/search')
    await expect(page.getByRole('heading', { name: 'Поиск' })).toBeVisible()
    await page.getByLabel('Поисковый запрос').fill('релиз')
    await page.getByRole('button', { name: 'Фильтры' }).click()
    await page.getByLabel('Пространство').fill('BASE')
    await page.getByLabel('Задача').fill('BASE-42')
    await page.getByLabel('Фаза').fill('implementation')
    await page.getByRole('button', { name: 'Применить' }).click()
    await expect
      .poll(() =>
        apiMocks.searchRequests.some(
          (query) =>
            query.includes('q=%D1%80%D0%B5%D0%BB%D0%B8%D0%B7') &&
            query.includes('space=BASE') &&
            query.includes('task_key=BASE-42') &&
            query.includes('phase_key=implementation'),
        ),
      )
      .toBe(true)
    await page.getByRole('button', { name: 'Сбросить фильтры' }).click()
    await expect(page.getByRole('link', { name: /Материал smoke-проверки фронта/ })).toBeVisible()
    await page.getByRole('link', { name: /Материал smoke-проверки фронта/ }).click()
    await expect(page).toHaveURL(`${baseURL}/evidence?id=${evidence.id}`)
    const selectedMaterial = page
      .getByRole('heading', { name: 'Выбранный материал' })
      .locator('..')
      .locator('..')
    await expect(selectedMaterial).toBeVisible()
    await expect(
      selectedMaterial.getByRole('link', { name: 'документ product-requirements' }),
    ).toBeVisible()
    await expect(selectedMaterial.getByRole('link', { name: 'задача BASE-42' })).toBeVisible()
    await expect(selectedMaterial.getByRole('link', { name: 'фаза implementation' })).toBeVisible()

    await gotoWiki(page, '/templates')
    await expect(page.getByRole('heading', { name: 'Шаблоны' })).toBeVisible()
    await page.getByRole('button', { name: 'Новый шаблон' }).click()
    await page.getByLabel('Название шаблона').fill('Шаблон релиза')
    await page.getByLabel('Тип документа').selectOption('release_note')
    await page.getByLabel('Markdown шаблона').fill('# Релиз\n\n## Проверки\n')
    await page.getByRole('button', { name: 'Создать шаблон' }).click()
    await expect
      .poll(() =>
        apiMocks.templateCreateRequests.some((request) => request.name === 'Шаблон релиза'),
      )
      .toBe(true)
    await expect(
      page.getByRole('button', { name: 'Показать содержимое шаблона Шаблон релиза' }),
    ).toBeVisible()

    await gotoWiki(page, '/users')
    await expect(page.getByRole('heading', { name: 'Профили Wiki', exact: true })).toBeVisible()
    await expect(page.getByRole('cell', { name: 'editor@example.com' })).toBeVisible()
    await expect(page.getByRole('cell', { name: 'Профиль доступен' }).last()).toBeVisible()

    await gotoWiki(page, '/settings')
    await expect(page.getByRole('heading', { name: 'Настройки' })).toBeVisible()
    await expect(page.getByText('PostgreSQL FTS')).toBeVisible()

    await gotoWiki(page, '/admin')
    await expect(page.getByRole('heading', { name: 'Администрирование' })).toBeVisible()
    await expect(page.getByRole('heading', { name: 'Состояние инстанса' })).toBeVisible()
    await expect(page.getByText('Файлы до 25 МБ')).toBeVisible()
  })
})
