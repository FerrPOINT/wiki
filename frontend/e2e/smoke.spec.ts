import { expect, test, type Route } from '@playwright/test'

const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? 'http://localhost:4173'

function routeJson(route: Route, body: unknown, status = 200) {
  return route.fulfill({
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  })
}

test.describe('wiki smoke', () => {
  test('login and navigate through wiki shell pages', async ({ page }) => {
    await page.route('**/api/v1/auth/login', (route) =>
      routeJson(route, {
        access_token: 'demo-token',
        token_type: 'Bearer',
        user_id: '00000000-0000-0000-0000-000000000001',
        email: 'demo@example.com',
      }),
    )
    await page.route('**/api/v1/auth/refresh', (route) =>
      routeJson(route, { access_token: 'demo-token', token_type: 'Bearer' }),
    )
    await page.route('**/api/v1/users/me', (route) =>
      routeJson(route, {
        id: '00000000-0000-0000-0000-000000000001',
        email: 'demo@example.com',
        username: 'demo',
        display_name: 'Demo User',
        is_system_admin: true,
      }),
    )
    await page.goto(`${baseURL}/login`)
    await page.getByRole('textbox').nth(0).fill('demo@example.com')
    await page.getByRole('textbox').nth(1).fill('demo')
    await page.getByRole('button', { name: /войти/i }).click()

    await expect(page).toHaveURL(`${baseURL}/`, { timeout: 10_000 })
    await expect(page.getByRole('heading', { name: 'Wiki' })).toBeVisible()

    await page.goto(`${baseURL}/spaces`)
    await expect(page.getByRole('heading', { name: 'Пространства' })).toBeVisible()
    await expect(page.getByText('Инженерия')).toBeVisible()

    await page.goto(`${baseURL}/documents/new`)
    await expect(page.getByRole('heading', { name: 'Новый документ' })).toBeVisible()

    await page.goto(`${baseURL}/tasks/SDLC-42`)
    await expect(page.getByRole('heading', { name: 'SDLC-42' })).toBeVisible()

    await page.goto(`${baseURL}/phases/implementation`)
    await expect(page.getByRole('heading', { name: 'Реализация' })).toBeVisible()

    await page.goto(`${baseURL}/search`)
    await expect(page.getByRole('heading', { name: 'Поиск' })).toBeVisible()
  })
})
