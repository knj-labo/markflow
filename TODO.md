# 🚀 Markflow Development Roadmap

| | |
| :--- | :--- |
| **Objective** | **Dramatically improve build performance** in Web frameworks (Astro, Next.js, Nuxt) and eliminate reliance on JavaScript dependencies. |
| **Current Status** | 🚧 Phase 1: Core Engine Refactoring |

## 🏃 In Progress

### Phase 1: Core Engine Refactoring
> **Goal:** Transition from prototype parser to `markdown-rs` for spec compliance and performance.

- [ ] **Core Parser Implementation**
    - [ ] Remove `pulldown-cmark` dependency (blocked by Event enum replacement)
    - [x] Introduce [`markdown-rs`](https://github.com/wooorm/markdown-rs) as the default parser
    - [x] Verify basic Markdown parsing functionality with new parser
    - [ ] Define internal `Event` enum and drop `pulldown-cmark` type usage
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

### Phase 2: Astro Integration (Content Layer API)
> **Goal:** Make the engine functional as an Astro v5 Loader.

- [ ] **Astro Loader Prototype**
    - [ ] Implement Loader readable from `src/content/config.ts`
    - [ ] Output data conforming to `RenderedContent` type
- [ ] **MDX Support (Step 1: Hybrid Bridge)**
    - [ ] Pass JSX blocks through as "raw text"
    - [ ] Verify handover to standard Astro pipeline
- [ ] **MDX Support (Step 2: Native Compiler)**
    - [ ] Introduce `swc_core`
    - [ ] Implement JSX code generation logic in Rust
    - [ ] Detect `client:*` directives and maintain hydration
- [ ] **Performance Measurement**
    - [ ] Compare build times for a 1000-page site (vs. Standard Astro)

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
