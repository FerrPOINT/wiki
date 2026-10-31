// Source/IO fixtures only: the formatter below is inert, not Prettier parity evidence.
import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import { mkdtemp, mkdir, readFile, realpath, rm, stat, utimes, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import test from 'node:test'

const source = await readFile(new URL('../generate-exports.mjs', import.meta.url), 'utf8')
const config = JSON.parse(await readFile(new URL('../../.prettierrc', import.meta.url), 'utf8'))
const formatter = `import { appendFileSync } from 'node:fs';
const record = value => appendFileSync(process.env.FIXTURE_CALLS, JSON.stringify(value) + '\\n');
export async function resolveConfig(filepath) { record({ kind: 'resolve', filepath }); return { semi: true }; }
export async function format(text, options) {
  record({ kind: 'format', options });
  const { filepath, ...values } = options;
  return '// inert fixture ' + JSON.stringify(values) + '\\n' + text;
}`

async function fixture(run) {
  const parent = await realpath(tmpdir())
  const owner = await realpath(await mkdtemp(join(parent, 'wiki-prepared-unit-')))
  const root = join(owner, 'frontend')
  assert.equal(dirname(owner), parent)
  for (const directory of ['scripts', 'src/api', 'node_modules/prettier']) {
    await mkdir(join(root, directory), { recursive: true })
  }
  const entry = join(root, 'scripts/generate-exports.mjs')
  const input = join(root, 'src/api/generated.ts')
  const output = join(root, 'src/api/generated-exports.ts')
  const settings = join(root, '.prettierrc')
  const callsPath = join(owner, 'calls.jsonl')
  await writeFile(entry, source)
  await writeFile(settings, JSON.stringify(config))
  await writeFile(
    input,
    'export interface components {\n    schemas: {\n        WikiDocument: { value: string };\n    };\n}\n',
  )
  await writeFile(
    join(root, 'node_modules/prettier/package.json'),
    JSON.stringify({ type: 'module', exports: './index.mjs' }),
  )
  await writeFile(join(root, 'node_modules/prettier/index.mjs'), formatter)
  const environment = { ...process.env, FIXTURE_CALLS: callsPath }
  delete environment.NODE_OPTIONS
  delete environment.NODE_PATH
  const generate = (args = ['--prepared']) =>
    execFileSync(process.execPath, [entry, ...args], {
      cwd: root,
      env: environment,
      stdio: ['ignore', 'pipe', 'pipe'],
      timeout: 5000,
    })
  const calls = async () => {
    try {
      return (await readFile(callsPath, 'utf8'))
        .trim()
        .split('\n')
        .filter(Boolean)
        .map((line) => JSON.parse(line))
    } catch (error) {
      if (error.code !== 'ENOENT') throw error
      return []
    }
  }
  try {
    await run({ owner, root, generate, output, settings, calls })
  } finally {
    assert.equal(await realpath(owner), owner)
    assert.equal(dirname(owner), parent)
    await rm(owner, { recursive: true, force: true })
  }
}

test('prepared bridge consumes declared root config without calling config discovery', async () => {
  await fixture(async ({ owner, root, generate, calls }) => {
    await writeFile(join(owner, '.prettierrc'), '{"semi":true}')
    await writeFile(join(root, 'src/api/.prettierrc'), '{"singleQuote":false}')
    generate()
    const recorded = await calls()
    assert.deepEqual(
      recorded.map((row) => row.kind),
      ['format'],
    )
    const { filepath, ...options } = recorded[0].options
    assert.equal(filepath, join(root, 'src/api/generated-exports.ts'))
    assert.deepEqual(options, config)
  })
})

test('manual bridge keeps the original resolveConfig contract', async () => {
  await fixture(async ({ generate, calls }) => {
    generate([])
    const recorded = await calls()
    assert.deepEqual(
      recorded.map((row) => row.kind),
      ['resolve', 'format'],
    )
    assert.equal(recorded[0].filepath, recorded[1].options.filepath)
    assert.equal(recorded[1].options.semi, true)
  })
})

test('prepared config rejects unknown, malformed and oversized inputs without output writes', async () => {
  await fixture(async ({ generate, output, settings, calls }) => {
    await writeFile(output, 'keep verified output')
    for (const [value, message] of [
      ['{', /SyntaxError/],
      ['[]', /must be a JSON object/],
      ['null', /must be a JSON object/],
      ['{"plugins":["external"]}', /unsupported options/],
      ['{"semi":"false"}', /unsupported options/],
      ['{"printWidth":0}', /unsupported options/],
      [' '.repeat(8193), /exceeds 8192 bytes/],
    ]) {
      await writeFile(settings, value)
      assert.throws(
        () => generate(),
        (error) => {
          assert.match(error.stderr.toString(), message)
          return true
        },
      )
      assert.equal(await readFile(output, 'utf8'), 'keep verified output')
    }
    assert.deepEqual(await calls(), [])
  })
})

test('missing config and unexpected prepared arguments fail without fallback', async () => {
  await fixture(async ({ generate, output, settings, calls }) => {
    await writeFile(output, 'keep verified output')
    assert.throws(
      () => generate(['--prepared', '--other']),
      (error) => {
        assert.match(error.stderr.toString(), /Prepared bridge accepts only/)
        return true
      },
    )
    await rm(settings)
    assert.throws(
      () => generate(),
      (error) => {
        assert.match(error.stderr.toString(), /ENOENT/)
        return true
      },
    )
    await mkdir(settings)
    assert.throws(
      () => generate(),
      (error) => {
        assert.match(error.stderr.toString(), /plain JSON file/)
        return true
      },
    )
    assert.equal(await readFile(output, 'utf8'), 'keep verified output')
    assert.deepEqual(await calls(), [])
  })
})

test('unchanged prepared generation preserves output identity and timestamp', async () => {
  await fixture(async ({ generate, output }) => {
    generate()
    await utimes(output, new Date(1000), new Date(1000))
    const before = await stat(output, { bigint: true })
    const bytes = await readFile(output)
    generate()
    const after = await stat(output, { bigint: true })
    assert.equal(after.mtimeNs, before.mtimeNs)
    assert.equal(after.ino, before.ino)
    assert.deepEqual(await readFile(output), bytes)
  })
})

test('changed config with an old timestamp updates the formatted result', async () => {
  await fixture(async ({ generate, output, settings }) => {
    generate()
    const before = await readFile(output)
    await writeFile(settings, JSON.stringify({ ...config, semi: !config.semi }))
    await utimes(settings, new Date(1), new Date(1))
    generate()
    assert.notDeepEqual(await readFile(output), before)
  })
})

test('prepared generation repairs a corrupt output from actual content', async () => {
  await fixture(async ({ generate, output }) => {
    generate()
    const expected = await readFile(output)
    await writeFile(output, Buffer.concat([expected, Buffer.from('corrupt')]))
    generate()
    assert.deepEqual(await readFile(output), expected)
  })
})
