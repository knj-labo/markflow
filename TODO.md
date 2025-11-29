# 🚀 Markflow Development Roadmap

| | |
| :--- | :--- |
| **Objective** | **Dramatically improve build performance** in Web frameworks (Astro, Next.js, Nuxt) and eliminate reliance on JavaScript dependencies. |
| **Current Status** | 🚧 Phase 1: Core Engine Refactoring |

## 🏃 In Progress

### Phase 1: Core Engine Refactoring
> **Goal:** Transition from prototype parser to `markdown-rs` for spec compliance and performance.

- [ ] **Core Parser Implementation**
    - [x] Remove `pulldown-cmark` dependency (HTML renderer implemented internally)
    - [x] Introduce [`markdown-rs`](https://github.com/wooorm/markdown-rs) as the default parser
    - [x] Verify basic Markdown parsing functionality with new parser
    - [x] Define internal `Event` enum (rendering bridge still converts to `pulldown-cmark` events)
- [ ] **WASM Bindings (Enhancement)**
    - [ ] Expand `crates/wasm` to support streaming APIs (currently minimal wrapper)

## ✅ Done

### Phase 0: Initialization
- [x] **Repository Setup**
    - [x] Set up repository and `cargo` workspace (`core`, `napi`, `wasm`)
    - [x] CI/CD pipeline configuration (GitHub Actions)

### Phase 1: Foundation Prototypes
- [x] **The Glue (PipeAdapter)**
    - [x] Implement adapter connecting `Iterator<Item=Event>` to `io::Write` (`crates/core/src/adapter.rs`)
    - [x] Benchmark stream connection (`benchmarks/stream_bench.rs`)
- [x] **Streaming Rewriter**
    - [x] Embed `lol_html`
    - [x] Implement basic tag rewriting (lazy images) (`crates/core/src/streaming_rewriter.rs`)
- [x] **NAPI Bindings**
    - [x] Set up `napi-rs`
    - [x] Expose basic functions (`parse`, `parse_with_options`)
    - [x] Verify Node.js interoperability

## 📋 Backlog

### Phase 2: Astro + Starlight MVP (Hybrid Loader + Vite)
> **Goal:** Ship Markflow as the MDX compiler for `withastro/docs` by combining the Content Layer loader with a pre-transform Vite plugin (Option 1 compiler path).

- [ ] **Content Loader Glue**
    - [ ] Mirror `src/content/config.ts` loader contract (glob discovery, slug generation, digest tracking)
    - [ ] Extract and store frontmatter for Starlight navigation/hero metadata in the DataStore
    - [ ] Keep non-Markflow entries compatible by reusing the default loader fallbacks
- [ ] **Vite Plugin Interception**
    - [ ] Publish `vite-plugin-markflow` with `enforce: 'pre'` to intercept `.mdx` before `@astrojs/mdx`
    - [ ] Call the NAPI `compile()` binding and emit JSX modules (source maps optional for MVP)
    - [ ] Verify coexistence with `@astrojs/starlight` auto-imported MDX integration
- [ ] **Rust Compiler Option 1**
    - [ ] YAML frontmatter extraction that emits `export const frontmatter`
    - [ ] Code-fence-aware import hoisting (state machine + multi-line support)
    - [ ] Markdown → JSX renderer with raw JSX passthrough to preserve components
    - [ ] `:::directive` → `<Aside>` transclusion plus auto-import injection
    - [ ] Heading slug generation (rehype-slug parity, including i18n) and `export const headings`
- [ ] **Hydration + Validation Strategy**
    - [ ] Smoke-test Tabs, FileTree, Steps, CardGrid to ensure hydration survives Option 1
    - [ ] Document known failure modes (e.g., malformed JSX) since compile-time validation is skipped
    - [ ] Provide fallback guidance for downgraded directive rendering if blockers appear
- [ ] **E2E with withastro/docs**
    - [ ] Wire repo fixture, run `npm run dev`, and load sample locales
    - [ ] Capture build and dev-server metrics vs. `@astrojs/mdx`
    - [ ] Log regressions and feed them back into backlog issues

### Phase 3: Universal Deployment (Nuxt & Next.js)
> **Goal:** Abstract framework-specific differences.

- [ ] **Nuxt (Content v3) Support**
    - [ ] Transformer Implementation (MDC syntax support)
    - [ ] Serialize to Minimal Tree (JSON AST)
    - [ ] Metadata Extraction (SQL storage structure)
- [ ] **Next.js (App Router / Turbopack) Support**
    - [ ] Implement `markflow-loader` (Webpack)
    - [ ] RSC Support (JSX generation for Server Components)
    - [ ] Unify build config via `unplugin`

### Phase 4: Ecosystem & Optimization
- [ ] **WASM Plugin System**: Load user-defined logic via WASM.
- [ ] **Rust-native Syntax Highlighting**: Integrate `syntect` or `tree-sitter`.
- [ ] **Turbopack Native Plugin**: Support Next.js official Rust plugin system.

## ⏱️ Astro MVP Work Breakdown

| Phase | Scope | Est. Effort |
| :--- | :--- | :--- |
| 1 | Environment + NAPI bridge (workspace bootstrap, napi-rs boilerplate, hello-world Vite plugin) | 3 days |
| 2 | Core compiler pieces (frontmatter extraction, code-fence-aware hoisting, baseline Markdown-to-JSX) | 5 days |
| 3 | Starlight-specific features (directive → Aside, slug generation, component passthrough validation) | 6 days |
| 4 | Integration + QA (withastro/docs wiring, hydration debugging, perf measurements) | 4 days |
| **Total** | **Prototype ready for docs site** | **≈18 person-days (3.6 weeks)** |
