/**
 * RSMD メイン実装（レンダリング・型・WASMローダーを集約）
 */

import matter from 'gray-matter'

// ===== 型定義 =====

export interface Options {
  gfmTables?: boolean
  gfmTasklists?: boolean
  footnotes?: boolean
  smartPunct?: boolean
}

export interface Heading {
  depth: number
  text: string
  slug: string
}

export interface RenderResult {
  html: string
  headings: Heading[]
}

export interface ParsedContent {
  frontmatter: Record<string, any>
  body: string
}

interface WasmOptions {
  gfm_tables: boolean
  gfm_tasklists: boolean
  footnotes: boolean
  smart_punct: boolean
}

export interface RsmdError extends Error {
  code: 'WASM_INIT' | 'RENDER' | 'PARSE'
  cause?: unknown
}

export const DEFAULT_OPTIONS: Required<Options> = {
  gfmTables: true,
  gfmTasklists: true,
  footnotes: true,
  smartPunct: true,
}

export function createError(message: string, code: RsmdError['code'], cause?: unknown): RsmdError {
  const error = new Error(message) as RsmdError
  error.name = 'RsmdError'
  error.code = code
  error.cause = cause
  return error
}

// ===== WASM ローダー =====

interface WasmModule {
  render_wasm: (source: string, options: WasmOptions) => RenderResult
  slugify_wasm: (text: string) => string
}

const wasmRuntime = (() => {
  let module: WasmModule | null = null
  let pendingInit: Promise<void> | null = null

  return Object.freeze({
    hasModule: () => module !== null,
    readModule: () => module,
    storeModule: (next: WasmModule) => {
      module = next
    },
    readInitPromise: () => pendingInit,
    storeInitPromise: (next: Promise<void>) => {
      pendingInit = next
    },
  })
})()

async function loadWasm(): Promise<void> {
  try {
    // TODO: 実際の WASM バンドル読込に差し替え
    // 例: const wasm = await import('../wasm/rsmd_core_bg.wasm');
    //      await wasm.default();
    //      wasmRuntime.storeModule(wasm);

    // 暫定モック: 仕様が固まったら削除
    wasmRuntime.storeModule({
      render_wasm: (source: string) => ({
        html: `<p>${source}</p>`,
        headings: [],
      }),
      slugify_wasm: (text: string) => text.toLowerCase().replace(/\s+/g, '-'),
    })
  } catch (error) {
    throw createError('Failed to load WASM module', 'WASM_INIT', error)
  }
}

export async function initWasm(): Promise<void> {
  if (wasmRuntime.hasModule()) return
  let promise = wasmRuntime.readInitPromise()
  if (!promise) {
    promise = loadWasm()
    wasmRuntime.storeInitPromise(promise)
  }
  await promise
}

function getWasmModule(): WasmModule {
  const module = wasmRuntime.readModule()
  if (!module) {
    throw createError(
      'WASM module not initialized. Use async render() or call initWasm() first.',
      'WASM_INIT'
    )
  }
  return module
}

function isWasmInitialized(): boolean {
  return wasmRuntime.hasModule()
}

/**
 * Markdownをレンダリング
 * フロントマターを分離し、本文のみをWASMに渡す
 */
export async function render(source: string, options: Options = {}): Promise<RenderResult> {
  try {
    // WASMモジュールの初期化（初回のみ）
    await initWasm()

    // フロントマターの分離
    const parsed = parseContent(source)

    // オプションのマージとWASM形式への変換
    const wasmOptions = prepareOptions(options)

    // WASMモジュールでレンダリング
    const result = renderWithWasm(parsed.body, wasmOptions)

    // TODO: オプショナル - JSポスト処理
    // 例: 外部リンクにrel="noopener"追加
    // 例: 見出しにアンカーリンク追加

    return result
  } catch (error) {
    if ((error as RsmdError).code) {
      throw error
    }
    throw createError('Render failed', 'RENDER', error)
  }
}

/**
 * 同期版render（WASMが初期化済みの場合のみ使用可能）
 */
export function renderSync(source: string, options: Options = {}): RenderResult {
  if (!isWasmInitialized()) {
    throw createError(
      'WASM not initialized. Use async render() or call initWasm() first.',
      'WASM_INIT'
    )
  }

  const parsed = parseContent(source)
  const wasmOptions = prepareOptions(options)
  return renderWithWasm(parsed.body, wasmOptions)
}

/**
 * フロントマター分離
 */
function parseContent(source: string): ParsedContent {
  try {
    const { data, content } = matter(source)
    return {
      frontmatter: data,
      body: content,
    }
  } catch (error) {
    throw createError('Failed to parse frontmatter', 'PARSE', error)
  }
}

/**
 * オプション準備（JS形式→WASM形式）
 */
function prepareOptions(options: Options): WasmOptions {
  const merged = { ...DEFAULT_OPTIONS, ...options }

  // TODO: プロパティ名のマッピング（camelCase → snake_case）
  return {
    gfm_tables: merged.gfmTables,
    gfm_tasklists: merged.gfmTasklists,
    footnotes: merged.footnotes,
    smart_punct: merged.smartPunct,
  }
}

/**
 * WASM呼び出し
 */
function renderWithWasm(body: string, options: WasmOptions): RenderResult {
  const module = getWasmModule()

  try {
    // TODO: WASM関数呼び出し
    const result = module.render_wasm(body, options)

    // TODO: 結果の検証と型変換
    if (!result || typeof result.html !== 'string' || !Array.isArray(result.headings)) {
      throw new Error('Invalid WASM response format')
    }

    return {
      html: result.html,
      headings: result.headings,
    }
  } catch (error) {
    throw createError('WASM render failed', 'RENDER', error)
  }
}

/**
 * slugify関数の公開（MDX経路との互換性用）
 */
export function slugify(text: string): string {
  // WASMが初期化済みなら使用、そうでなければフォールバック
  if (isWasmInitialized()) {
    const module = getWasmModule()
    return module.slugify_wasm(text)
  }

  // TODO: JSフォールバック実装
  return text
    .toLowerCase()
    .replace(/\s+/g, '-')
    .replace(/[^\w\u3000-\u303F\u3040-\u309F\u30A0-\u30FF\u4E00-\u9FFF\uAC00-\uD7AF-]/g, '')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '')
}
