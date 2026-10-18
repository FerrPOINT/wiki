// @vitest-environment node

import { mkdtempSync, mkdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { tmpdir } from 'node:os'
import { basename, dirname, join, resolve } from 'node:path'
import { expect, it } from 'vitest'

type Source = { base: string; pattern: string; negated: boolean }

it('keeps production and library classes while test fixtures stay outside CSS inputs', async () => {
  const require = createRequire(join(process.cwd(), 'package.json'))
  const { compile } = require('tailwindcss') as typeof import('tailwindcss')
  const pluginRequire = createRequire(require.resolve('@tailwindcss/vite'))
  const { Scanner } = pluginRequire('@tailwindcss/oxide') as {
    Scanner: new (options: { sources: Source[] }) => { scan(): string[] }
  }
  const stylesheets = new Map([
    ['tailwindcss', require.resolve('tailwindcss/index.css')],
    ['@pdlc/ui/tokens.css', require.resolve('@pdlc/ui/tokens.css')],
  ])
  const policy = readFileSync(resolve(process.cwd(), 'src/index.css'), 'utf8')
  const fixture = mkdtempSync(join(tmpdir(), 'wiki-tailwind-unit-policy-'))
  try {
    const files = {
      '.gitignore': 'node_modules/\n',
      'src/main.tsx': '<div className="p-[73px]" />',
      'src/contest.tsx': '<div className="mx-[19px]" />',
      'src/main.test.tsx': '<div className="h-[937px]" />',
      'src/main.spec.tsx': '<div className="max-h-[941px]" />',
      'e2e/fixtures.ts': '<div className="left-[947px]" />',
      'src/test/fixtures.ts': '<div className="right-[953px]" />',
      'node_modules/@pdlc/ui/src/button.tsx': '<button className="w-[211px]" />',
      'node_modules/@pdlc/ui/src/button.test.tsx': '<button className="m-[997px]" />',
      'node_modules/@pdlc/ui/src/button.spec.tsx': '<button className="top-[967px]" />',
    }
    for (const [name, value] of Object.entries(files)) {
      const path = join(fixture, name)
      mkdirSync(dirname(path), { recursive: true })
      writeFileSync(path, value)
    }
    const scan = async (css: string) => {
      const compiled = await compile(css, {
        base: join(fixture, 'src'),
        loadStylesheet: async (id, base) => {
          const file = id.startsWith('.') ? resolve(base, id) : stylesheets.get(id)
          if (!file) throw new Error(`Unqualified stylesheet import: ${id}`)
          return { path: file, base: dirname(file), content: readFileSync(file, 'utf8') }
        },
      })
      const sources: Source[] = [...compiled.sources]
      if (compiled.root !== 'none') {
        sources.unshift(
          compiled.root
            ? { ...compiled.root, negated: false }
            : { base: fixture, pattern: '**/*', negated: false },
        )
      }
      return new Scanner({ sources }).scan()
    }
    const before = await scan(policy.replace(/@source\s+not\s+[^;]+;/g, ''))
    const after = await scan(policy)
    const production = ['p-[73px]', 'mx-[19px]', 'w-[211px]']
    const tests = [
      'h-[937px]',
      'm-[997px]',
      'max-h-[941px]',
      'left-[947px]',
      'right-[953px]',
      'top-[967px]',
    ]
    expect(before).toEqual(expect.arrayContaining([...production, ...tests]))
    expect(after).toEqual(expect.arrayContaining(production))
    for (const candidate of tests) expect(after).not.toContain(candidate)
    const css = async (candidates: string[]) =>
      (await compile('@tailwind utilities;')).build(candidates)
    const oldCss = await css(before)
    const nextCss = await css(after)
    for (const pixels of ['937px', '997px', '941px', '947px', '953px', '967px']) {
      expect(oldCss).toContain(pixels)
      expect(nextCss).not.toContain(pixels)
    }
    for (const pixels of ['73px', '19px', '211px']) expect(nextCss).toContain(pixels)
  } finally {
    expect(dirname(realpathSync(fixture))).toBe(realpathSync(tmpdir()))
    expect(basename(fixture)).toMatch(/^wiki-tailwind-unit-policy-/)
    rmSync(fixture, { recursive: true })
  }
})
