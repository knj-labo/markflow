#!/usr/bin/env node
import { readFile } from 'node:fs/promises'
import { basename, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { performance } from 'node:perf_hooks'

import { parse, parseWithOptions, parseWithStats } from '../crates/napi/index.js'

const __dirname = fileURLToPath(new URL('.', import.meta.url))
const repoRoot = resolve(__dirname, '..')
const userArgs = process.argv.slice(2).filter((arg) => arg !== '--')
const [inputArg = 'fixtures/markdown/hello.md'] = userArgs
const inputPath = resolve(repoRoot, inputArg)

const markdown = await readFile(inputPath, 'utf8')
console.log(`🚀 Running markflow-napi smoke test for ${basename(inputPath)}`)

const start = performance.now()
const html = parse(markdown)
const elapsed = performance.now() - start
console.log(`✅ parse() returned ${html.length} bytes in ${elapsed.toFixed(3)}ms`)

const custom = parseWithOptions(markdown, { enforceImgLoadingLazy: false })
console.log(`✅ parseWithOptions() toggled lazy attr: ${(custom.includes('loading="lazy"') ? 'on' : 'off')}`)

const stats = parseWithStats(markdown)
console.log(`✅ parseWithStats() reports ${stats.processingTimeMs.toFixed(3)}ms with ${stats.html.length} bytes`)

console.log('\n📝 HTML preview:\n', html.slice(0, 160), html.length > 160 ? '…' : '')
