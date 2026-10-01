import { mkdirSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import AxeBuilder from '@axe-core/playwright'
import { expect, test } from '@playwright/test'

test.skip(process.env.SDLC_LIVE_QA !== '1', 'Requires live Wiki and Central Auth')
test.skip(({ browserName }) => browserName !== 'chromium', 'Single browser live acceptance')
test.use({ trace: 'off', hasTouch: true })

const account =
  process.env.SDLC_LIVE_QA === '1'
    ? (JSON.parse(
        readFileSync(
          process.env.SDLC_QA_SESSION_FILE ??
            fileURLToPath(
              new URL('../../../services-base/deploy/.local/qa-session.json', import.meta.url),
            ),
          'utf8',
        ),
      ) as { email: string; password: string })
    : { email: '', password: '' }
const base = process.env.PLAYWRIGHT_BASE_URL ?? 'http://localhost:7732'
const screenshots = fileURLToPath(
  new URL('../../../.local/screenshots/wiki-detail-layout/', import.meta.url),
)

test('Wiki details use real shared rails or intentional wide content with scoped links', async ({
  page,
  request,
}) => {
  test.setTimeout(900_000)
  mkdirSync(screenshots, { recursive: true })
  const login = await request.post('http://localhost:7701/auth/login', {
    data: { email: account.email, password: account.password },
  })
  expect(login.status()).toBe(200)
  const { access_token } = (await login.json()) as { access_token: string }
  const headers = { Authorization: `Bearer ${access_token}` }
  const api = `${base}/api/v1`
  const space = `QA${Date.now().toString(36).toUpperCase()}`
  const taskKey = `${space}-42`
  const phaseKey = 'qa-layout'
  let spaceCreated = false
  let documentId = ''
  const errors: string[] = []
  const mutations: string[] = []
  try {
    const createdSpace = await request.post(`${api}/spaces`, {
      headers,
      data: { key: space, name: `QA ${space} layout`, description: 'Isolated detail acceptance' },
    })
    spaceCreated = createdSpace.ok()
    expect(spaceCreated).toBeTruthy()
    const createdDocument = await request.post(`${api}/spaces/${space}/documents`, {
      headers,
      data: {
        title: `QA ${space} документ`,
        content_markdown: '## Проверенная геометрия\n\nТестовое опубликованное содержание.',
        task_key: taskKey,
        phase_key: phaseKey,
      },
    })
    if (createdDocument.ok()) documentId = ((await createdDocument.json()) as { id: string }).id
    expect(createdDocument.ok()).toBeTruthy()
    const first = await request.post(`${api}/documents/${documentId}/publish`, {
      headers,
      data: { summary: 'QA первая ревизия' },
    })
    expect(first.ok()).toBeTruthy()
    const firstId = ((await first.json()) as { id: string }).id
    const edited = await request.put(`${api}/documents/${documentId}/draft`, {
      headers,
      data: {
        title: `QA ${space} документ`,
        content_markdown: '## Проверенная геометрия\n\nВторая тестовая ревизия.',
      },
    })
    expect(edited.ok()).toBeTruthy()
    const second = await request.post(`${api}/documents/${documentId}/publish`, {
      headers,
      data: { base_revision_id: firstId, summary: 'QA вторая ревизия' },
    })
    expect(second.ok()).toBeTruthy()
    const evidence = await request.post(`${api}/evidence`, {
      headers,
      data: {
        space,
        title: 'QA подтверждение layout',
        evidence_type: 'external_url',
        url: 'https://example.test/qa/layout',
        document_id: documentId,
        task_key: taskKey,
        phase_key: phaseKey,
      },
    })
    expect(evidence.ok()).toBeTruthy()

    await page.goto(`${base}/documents/${documentId}`)
    await page.getByLabel('Email').fill(account.email)
    await page.getByLabel('Пароль').fill(account.password)
    await page.getByRole('button', { name: 'Войти', exact: true }).click()
    await expect(page.getByRole('complementary', { name: 'Контекст документа' })).toBeVisible()
    page.on('pageerror', (error) => errors.push(`page: ${error.message}`))
    page.on('console', (message) => {
      if (message.type() === 'error') errors.push(`console: ${message.text()}`)
    })
    page.on('requestfailed', (failed) => {
      const reason = failed.failure()?.errorText ?? 'unknown'
      if (!reason.includes('ERR_ABORTED'))
        errors.push(`${failed.method()} ${new URL(failed.url()).pathname}: ${reason}`)
    })
    page.on('response', (response) => {
      if (response.url().includes('/api/v1/') && response.status() >= 400)
        errors.push(`${response.status()} ${new URL(response.url()).pathname}`)
    })
    page.on('request', (outgoing) => {
      if (outgoing.url().includes('/api/v1/') && outgoing.method() !== 'GET')
        mutations.push(`${outgoing.method()} ${new URL(outgoing.url()).pathname}`)
    })
    const routes = [
      ['document-read', `/documents/${documentId}`, 'Контекст документа'],
      ['document-edit', `/documents/${documentId}`, 'Контекст документа'],
      ['task', `/tasks/${taskKey}?space=${space}`, 'Фазы материалов'],
      ['phase', `/phases/${phaseKey}?space=${space}`, null],
    ] as const
    for (const theme of ['light', 'gray', 'dark']) {
      await page.evaluate((value) => localStorage.setItem('theme', value), theme)
      for (const [route, path, label] of routes) {
        await page.goto(`${base}${path}`)
        await expect(page.locator('html')).toHaveAttribute('data-theme', theme)
        if (route === 'document-edit') await page.getByRole('button', { name: 'Правка' }).click()
        const layout = page.locator(
          label ? '.page-split' : '[data-dossier-layout="parallel-content"]',
        )
        await expect(layout).toBeVisible()
        const frame = page.locator('main [data-page-layout]').first()
        await expect(frame).toHaveAttribute(
          'data-page-layout',
          label ? 'detail-with-aside' : 'wide',
        )
        for (const [width, height] of [
          [375, 812],
          [768, 1024],
          [1023, 800],
          [1024, 800],
          [1279, 800],
          [1280, 800],
          [1440, 900],
          [1920, 1080],
          [2560, 1440],
        ]) {
          await page.setViewportSize({ width, height })
          await page.evaluate(() => window.scrollTo(0, 0))
          const dimensions = await layout.evaluate((element) => {
            const primary = element.firstElementChild!.getBoundingClientRect()
            const secondary = element.lastElementChild!.getBoundingClientRect()
            return {
              primary: {
                x: primary.x,
                y: primary.y,
                width: primary.width,
                right: primary.right,
                bottom: primary.bottom,
              },
              secondary: { x: secondary.x, y: secondary.y, width: secondary.width },
              gap: parseFloat(getComputedStyle(element).columnGap),
            }
          })
          const split = width >= (label ? 1024 : 1280)
          if (split) {
            expect(dimensions.secondary.width).toBeCloseTo(
              label ? 320 : dimensions.primary.width,
              0,
            )
            if (!label) expect(dimensions.secondary.width).toBeGreaterThan(320)
            expect(dimensions.secondary.x - dimensions.primary.right).toBeCloseTo(dimensions.gap, 0)
            expect(dimensions.secondary.y).toBeCloseTo(dimensions.primary.y, 0)
          } else {
            expect(dimensions.secondary.width).toBeCloseTo(dimensions.primary.width, 0)
            expect(dimensions.secondary.x).toBeCloseTo(dimensions.primary.x, 0)
            expect(dimensions.secondary.y).toBeGreaterThanOrEqual(dimensions.primary.bottom)
          }
          if (route === 'document-read') {
            const bodyWidth = await layout
              .locator('.wiki-rendered')
              .evaluate((el) => el.getBoundingClientRect().width)
            expect(bodyWidth).toBeLessThanOrEqual(760)
          }
          expect(
            await page.evaluate(
              () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
            ),
            `${route} ${theme} ${width} overflow`,
          ).toBeLessThanOrEqual(1)
          const audit = await new AxeBuilder({ page }).analyze()
          expect(
            audit.violations
              .filter((issue) => issue.impact === 'serious' || issue.impact === 'critical')
              .map((issue) => ({ id: issue.id, targets: issue.nodes.map((node) => node.target) })),
            `${route} ${theme} ${width} axe`,
          ).toEqual([])
          await page.screenshot({
            path: `${screenshots}/${route}-${theme}-${width}.png`,
            fullPage: true,
            animations: 'disabled',
          })
        }
        if (route === 'document-read') {
          for (const width of [375, 2560]) {
            await page.setViewportSize({ width, height: 812 })
            const open = page.getByRole('button', { name: 'Открыть ревизию 2', exact: true })
            if (width === 375) await open.tap()
            else {
              await open.focus()
              await page.keyboard.press('Enter')
            }
            const dialog = page.getByRole('dialog', { name: 'Снимок ревизии' })
            await expect(dialog.getByText('Вторая тестовая ревизия.')).toBeVisible()
            await page.keyboard.press('Escape')
            await expect(dialog).toBeHidden()
            await expect(open).toBeFocused()
          }
          for (const [name, destination] of [
            [taskKey, 'tasks'],
            [phaseKey, 'phases'],
          ]) {
            await page.getByRole('link', { name, exact: true }).click()
            await expect(page).toHaveURL(`${base}/${destination}/${name}?space=${space}`)
            await expect(
              page.getByRole('heading', {
                name: destination === 'tasks' ? 'Документы задачи' : 'Документы фазы',
              }),
            ).toBeVisible()
            await page.goto(`${base}/documents/${documentId}`)
            await expect(
              page.getByRole('complementary', { name: 'Контекст документа' }),
            ).toBeVisible()
          }
        }
      }
    }
    expect(
      mutations,
      'Read/edit layout, revision snapshots and scoped navigation must not write data',
    ).toEqual([])
    expect(errors).toEqual([])
  } finally {
    if (documentId) {
      const archived = await request.post(`${api}/documents/${documentId}/archive`, { headers })
      expect(archived.ok()).toBeTruthy()
    }
    if (spaceCreated) {
      const archived = await request.post(`${api}/spaces/${space}/archive`, { headers })
      expect(archived.ok()).toBeTruthy()
    }
  }
})
