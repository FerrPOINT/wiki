// Evidence screenshots for Wiki README and docs.
// Usage: node scripts/shoot-evidence.mjs
import { mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from '@playwright/test'

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..', '..')
const OUT = join(ROOT, 'docs', 'screenshots')
const BASE = process.env.PLAYWRIGHT_BASE_URL ?? 'http://127.0.0.1:4174'
const now = '2026-08-31T10:00:00Z'

mkdirSync(OUT, { recursive: true })

const authState = {
  state: {
    token: 'screenshot-token',
    userId: '00000000-0000-0000-0000-000000000001',
    email: 'admin@example.test',
    username: 'admin',
    displayName: 'Администратор',
  },
  version: 0,
}

const user = {
  id: authState.state.userId,
  email: authState.state.email,
  username: authState.state.username,
  display_name: authState.state.displayName,
  role: 'admin',
  is_system_admin: true,
  active: true,
}
const editorUser = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'editor@example.test',
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
  space_key: 'SDLC',
  document_id: 'product-requirements',
  task_key: 'SDLC-42',
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

function escapeHtml(value) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
}

function renderMockMarkdown(markdown) {
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

const revision = {
  id: 'revision-product-requirements-1',
  document_id: 'product-requirements',
  version: 1,
  title: 'Требования к Wiki MVP',
  body_markdown: documentBodyMarkdown,
  body_html: renderMockMarkdown(documentBodyMarkdown),
  summary: 'Исходные требования MVP',
  author_id: user.id,
  published_at: now,
}

const document = {
  id: 'product-requirements',
  space_key: 'SDLC',
  parent_id: null,
  slug: 'product-requirements',
  title: 'Требования к Wiki MVP',
  document_type: 'requirements',
  status: 'published',
  can_edit: true,
  body_markdown: revision.body_markdown,
  body_html: revision.body_html,
  draft_markdown: revision.body_markdown,
  current_revision: revision,
  task_keys: ['SDLC-42'],
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
  space_key: 'SDLC',
  task_key: 'SDLC-42',
  title: document.title,
  document_count: 1,
  evidence_count: 2,
  documents: [documentSummary],
  evidence: [evidence, fileEvidence],
}

const phase = {
  space_key: 'SDLC',
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
const templates = [
  {
    id: 'requirements',
    name: 'Требования',
    document_type: 'requirements',
    body_markdown: '# Требования\n\n## Контекст\n\n## Решения\n\n## Проверки\n',
  },
  {
    id: 'research-note',
    name: 'Исследование',
    document_type: 'research_note',
    body_markdown: '# Исследование\n\n## Контекст\n\n## Варианты\n\n## Решение\n',
  },
]

const shots = [
  { name: '01-login.png', path: '/login', title: 'Login' },
  { name: '02-register.png', path: '/register', title: 'Register' },
  { name: '03-dashboard.png', path: '/', title: 'Dashboard' },
  { name: '04-spaces.png', path: '/spaces', title: 'Spaces' },
  { name: '05-document-compose.png', path: '/documents/new', title: 'Document compose' },
  {
    name: '06-document-view.png',
    path: '/documents/product-requirements',
    title: 'Document view',
    openRevision: true,
  },
  { name: '07-task-dossiers.png', path: '/tasks', title: 'Task pages' },
  { name: '08-task-dossier-detail.png', path: '/tasks/SDLC-42', title: 'Task page detail' },
  { name: '09-phase-dossiers.png', path: '/phases', title: 'Phase pages' },
  {
    name: '10-phase-dossier-detail.png',
    path: '/phases/implementation',
    title: 'Phase page detail',
  },
  { name: '11-evidence.png', path: '/evidence', title: 'Evidence' },
  { name: '12-templates.png', path: '/templates', title: 'Templates' },
  { name: '13-audit-log.png', path: '/audit-log', title: 'Audit log' },
  { name: '14-users.png', path: '/users', title: 'Users' },
  { name: '15-settings.png', path: '/settings', title: 'Settings' },
  { name: '16-search.png', path: '/search', title: 'Search' },
  { name: '17-admin.png', path: '/admin', title: 'Administration' },
  { name: 'm-dashboard.png', path: '/', title: 'Dashboard mobile', mobile: true },
  { name: 'm-spaces.png', path: '/spaces', title: 'Spaces mobile', mobile: true },
  {
    name: 'm-document-view.png',
    path: '/documents/product-requirements',
    title: 'Document view mobile',
    mobile: true,
  },
  {
    name: 'm-task-dossier.png',
    path: '/tasks/SDLC-42',
    title: 'Task page mobile',
    mobile: true,
  },
  { name: 'm-search.png', path: '/search', title: 'Search mobile', mobile: true },
]

function routeJson(route, body, status = 200) {
  return route.fulfill({
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  })
}

async function installApiMocks(page) {
  await page.route('**/api/v1/**', (route) => {
    const request = route.request()
    const url = new URL(request.url())
    const path = url.pathname.replace('/api/v1', '')
    const method = request.method()

    if (method === 'POST' && path === '/auth/login') {
      return routeJson(route, {
        access_token: 'screenshot-token',
        refresh_token: 'screenshot-refresh',
        token_type: 'Bearer',
        user_id: user.id,
        email: user.email,
        username: user.username,
        display_name: user.display_name,
      })
    }
    if (method === 'POST' && path === '/auth/register') {
      return routeJson(route, {
        access_token: 'screenshot-token',
        refresh_token: 'screenshot-refresh',
        token_type: 'Bearer',
        user_id: user.id,
        email: 'new@example.test',
        username: 'new-user',
        display_name: 'Новый пользователь',
      })
    }
    if (method === 'POST' && path === '/auth/refresh') {
      return routeJson(route, {
        access_token: 'screenshot-token',
        refresh_token: 'screenshot-refresh',
      })
    }
    if (method === 'POST' && path === '/auth/logout') return route.fulfill({ status: 204 })
    if (method === 'GET' && path === '/users/me') return routeJson(route, user)
    if (method === 'GET' && path === '/users')
      return routeJson(route, { users: [user, editorUser] })
    if (method === 'PUT' && path.startsWith('/users/')) {
      const userId = decodeURIComponent(path.split('/').pop() ?? '')
      const body = request.postDataJSON()
      return routeJson(route, {
        ...(userId === editorUser.id ? editorUser : user),
        role: body.role ?? 'user',
        is_system_admin: body.role === 'admin',
        active: body.active ?? true,
      })
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
      return routeJson(route, document)
    }
    if (method === 'GET' && path === '/documents/product-requirements/revisions') {
      return routeJson(route, { revisions: [revision] })
    }
    if (
      method === 'GET' &&
      path === '/documents/product-requirements/revisions/revision-product-requirements-1'
    ) {
      return routeJson(route, revision)
    }
    if (method === 'GET' && path === '/spaces/SDLC/tasks') {
      return routeJson(route, { tasks: [task] })
    }
    if (method === 'GET' && path === '/spaces/SDLC/tasks/SDLC-42') {
      return routeJson(route, task)
    }
    if (method === 'GET' && path === '/spaces/SDLC/phases') {
      return routeJson(route, { phases: [phase] })
    }
    if (method === 'GET' && path === '/spaces/SDLC/phases/implementation') {
      return routeJson(route, phase)
    }
    if (method === 'GET' && path === '/evidence') {
      return routeJson(route, { evidence: [evidence, fileEvidence] })
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
        templates,
      })
    }
    if (method === 'POST' && path === '/templates') {
      const body = request.postDataJSON()
      return routeJson(
        route,
        {
          id: body.name.toLowerCase().replace(/\s+/g, '-'),
          name: body.name,
          document_type: body.document_type,
          body_markdown: body.body_markdown,
        },
        201,
      )
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
            request_id: 'mock-request',
            created_at: now,
          },
        ],
      })
    }
    if (method === 'GET' && path === '/search') {
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
}

const browser = await chromium.launch()

async function shoot(shot) {
  const context = await browser.newContext({
    viewport: shot.mobile ? { width: 375, height: 812 } : { width: 1920, height: 1080 },
    deviceScaleFactor: 1,
    locale: 'ru-RU',
  })
  const page = await context.newPage()
  await page.addInitScript((state) => {
    localStorage.setItem('theme', 'dark')
    localStorage.setItem('wiki-auth', JSON.stringify(state))
  }, authState)
  await installApiMocks(page)

  try {
    await page.goto(`${BASE}${shot.path}`, { waitUntil: 'networkidle', timeout: 30_000 })
    if (shot.openRevision) {
      await page.getByRole('button', { name: 'Открыть' }).first().click()
      await page.getByRole('heading', { name: 'Снимок ревизии' }).waitFor({ timeout: 5_000 })
    }
    await page.waitForTimeout(1000)
    await page.screenshot({ path: join(OUT, shot.name), fullPage: true })
    console.log(`shot ${shot.name} ${shot.path} ${shot.title}`)
  } finally {
    await context.close()
  }
}

for (const shot of shots) await shoot(shot)

await browser.close()
console.log(`done: ${shots.length}`)
