import { spawn } from 'node:child_process'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const viteBin = join(root, 'node_modules', 'vite', 'bin', 'vite.js')

process.env.VITE_API_BASE_URL ??= 'http://127.0.0.1:3456/api/v1'
const port = process.env.PLAYWRIGHT_PREVIEW_PORT ?? '4174'

const child = spawn(process.execPath, [viteBin, 'preview', '--port', port, '--strictPort'], {
  cwd: root,
  env: process.env,
  stdio: 'inherit',
})

function stop(signal) {
  if (!child.killed) child.kill(signal)
}

process.on('SIGINT', () => stop('SIGINT'))
process.on('SIGTERM', () => stop('SIGTERM'))

child.on('exit', (code, signal) => {
  if (signal) process.exit(0)
  process.exit(code ?? 0)
})
