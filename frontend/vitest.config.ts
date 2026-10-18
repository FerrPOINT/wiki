import { defineConfig, mergeConfig } from 'vitest/config'
import viteConfig from './vite.config'

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      globals: true,
      environment: 'jsdom',
      maxWorkers: 4,
      // Threads are qualified on the pinned Windows toolchain; other hosts keep forks.
      pool: process.platform === 'win32' ? 'threads' : 'forks',
      isolate: true,
      setupFiles: ['./src/test/setup.ts'],
      include: ['src/**/*.test.{ts,tsx}'],
    },
  }),
)
