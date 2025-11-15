import { defineConfig } from 'vitest/config'
import wasm from 'vite-plugin-wasm'
import { fileURLToPath } from 'node:url'
import { dirname, resolve } from 'node:path'

const rootDir = dirname(fileURLToPath(new URL('../../package.json', import.meta.url)))
const suiteRoot = fileURLToPath(new URL('./', import.meta.url))
const rsmdEntry = resolve(rootDir, 'packages/rsmd/src/index.ts')

export default defineConfig({
  test: {
    root: suiteRoot,
    environment: 'node',
    include: ['**/*.test.ts'],
    reporters: 'default',
    clearMocks: true,
  },
  plugins: [wasm()],
  resolve: {
    alias: {
      '@jp-knj/rsmd': rsmdEntry,
    },
  },
})
