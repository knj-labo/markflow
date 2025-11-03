/**
 * @jp-knj/rsmd パッケージの公開API
 */

export {
  render,
  renderSync,
  slugify,
  initWasm,
  createError,
  DEFAULT_OPTIONS,
} from './render.js'

export type {
  Options,
  RenderResult,
  Heading,
  ParsedContent,
  RsmdError,
} from './render.js'

// バージョン情報（暫定値）
export const VERSION = '0.0.1'
