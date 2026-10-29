import assert from 'node:assert/strict'
import { mkdtemp, mkdir, readFile, stat, utimes, writeFile, rm, realpath } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { pathToFileURL } from 'node:url'
import test from 'node:test'

const source = await readFile(new URL('../generate-exports.mjs', import.meta.url), 'utf8')
const formatter = import.meta.resolve('prettier')
const options = await readFile(new URL('../../.prettierrc', import.meta.url), 'utf8')

async function fixture(run) {
  const temporary = await realpath(tmpdir())
  const root = await realpath(await mkdtemp(join(temporary, 'wiki-bridge-unit-')))
  assert.equal(dirname(root), temporary)
  const scripts = join(root, 'scripts'),
    api = join(root, 'src/api')
  await mkdir(scripts, { recursive: true })
  await mkdir(api, { recursive: true })
  const entry = join(scripts, 'generate-exports.mjs'),
    input = join(api, 'generated.ts'),
    output = join(api, 'generated-exports.ts')
  assert.equal(source.split("from 'prettier'").length, 2)
  await writeFile(entry, source.replace("from 'prettier'", 'from ' + JSON.stringify(formatter)))
  await writeFile(join(root, '.prettierrc'), options)
  let invocation = 0
  const generate = async (prepared = false) => {
    const previous = process.argv
    if (prepared) process.argv = [previous[0], entry, '--prepared']
    try {
      await import(pathToFileURL(entry).href + '?unit=' + invocation++)
    } finally {
      process.argv = previous
    }
  }
  const schema = (name) =>
    `export interface components {\n    schemas: {\n        ${name}: { value: string };\n    };\n}\nexport interface paths { unrelated: { parameters: string }; }\n`
  try {
    await run({ generate, input, output, schema, api })
  } finally {
    assert.equal(await realpath(root), root)
    assert.equal(dirname(root), temporary)
    await rm(root, { recursive: true, force: true })
  }
}

test('bridge keeps schema aliases without treating path parameters as schemas', async () => {
  await fixture(async ({ generate, input, output, schema }) => {
    await writeFile(input, schema('WikiDocument'))
    await generate()
    const result = await readFile(output, 'utf8')
    assert.match(result, /export type WikiDocument = components\['schemas'\]\['WikiDocument'\]/)
    assert.doesNotMatch(result, /export type unrelated|export type parameters/)
  })
})

test('prepared root options match the real formatter and ignore a nearer manual config', async () => {
  await fixture(async ({ generate, input, output, schema, api }) => {
    await writeFile(input, schema('WikiDocument'))
    await generate()
    const baseline = await readFile(output)
    await generate(true)
    assert.deepEqual(await readFile(output), baseline)
    await writeFile(join(api, '.prettierrc'), JSON.stringify({ semi: true }))
    const prettier = await import(formatter)
    await prettier.clearConfigCache()
    await generate()
    assert.notDeepEqual(await readFile(output), baseline)
    await generate(true)
    assert.deepEqual(await readFile(output), baseline)
  })
})

test('identical generation preserves output bytes, inode and mtime', async () => {
  await fixture(async ({ generate, input, output, schema }) => {
    await writeFile(input, schema('WikiDocument'))
    await generate()
    await utimes(output, new Date(1000), new Date(1000))
    const before = await stat(output, { bigint: true }),
      bytes = await readFile(output)
    await generate()
    const after = await stat(output, { bigint: true })
    assert.equal(after.mtimeNs, before.mtimeNs)
    assert.equal(after.ino, before.ino)
    assert.deepEqual(await readFile(output), bytes)
  })
})

test('older input timestamp cannot conceal changed schema aliases', async () => {
  await fixture(async ({ generate, input, output, schema }) => {
    await writeFile(input, schema('Before'))
    await generate()
    await writeFile(input, schema('After'))
    await utimes(input, new Date(1), new Date(1))
    await generate()
    const result = await readFile(output, 'utf8')
    assert.match(result, /export type After/)
    assert.doesNotMatch(result, /export type Before/)
  })
})

test('missing and prefix-corrupted outputs are repaired from actual inputs', async () => {
  await fixture(async ({ generate, input, output, schema }) => {
    await writeFile(input, schema('WikiDocument'))
    await generate()
    const expected = await readFile(output)
    await writeFile(output, Buffer.concat([expected, Buffer.alloc(65536, 120)]))
    await generate()
    assert.deepEqual(await readFile(output), expected)
    await rm(output)
    await generate()
    assert.deepEqual(await readFile(output), expected)
  })
})
