import { chromium } from '@playwright/test'

const output = process.argv[2]
const baseUrl = process.env.WIKI_EVIDENCE_BASE_URL ?? 'http://127.0.0.1:7732'
if (!output) throw new Error('usage: node capture-wiki-login.mjs OUTPUT')

const browser = await chromium.launch({ headless: true })
const page = await browser.newPage({ viewport: { width: 375, height: 812 } })
await page.goto(`${baseUrl}/login`, { waitUntil: 'networkidle' })
for (const input of await page.locator('input').all()) await input.fill('')
await page.locator('body').click({ position: { x: 8, y: 8 } })
await page.screenshot({ path: output, fullPage: false })
await browser.close()
