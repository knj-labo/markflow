## 🚀 Markflow Development Master Plan

| | |
| :--- | :--- |
| **Overview** | Next-generation streaming Markdown/MDX engine built with **Rust**. |
| **Objective** | **Dramatically improve build performance** in Web frameworks (Astro, Next.js, Nuxt) and eliminate reliance on JavaScript dependencies. |
| **Current Status** | Planning Phase (Phase 0) |

-----

## 1. Core Architecture Design

### Basic Principles

  * Zero-Copy Streaming: Avoid holding the entire document in memory from input to output.
  * Hybrid Distribution: Provide NAPI-RS for build-time operations and WASM for runtime usage.
  * Modular Pipeline: Input $\rightarrow$ Parser (Pull) $\rightarrow$ Adapter (Glue) $\rightarrow$ Rewriter (Push) $\rightarrow$ Output.

### Data Flow Diagram (Mermaid)

```mermaid
graph TD
    Input[Input Stream] -->|Bytes| Parser[Core Parser (markdown-rs)]
    Parser -->|Events| Processor[Event Processor (Directives)]
    Processor -->|Modified Events| Glue[The Glue (PipeAdapter)]
    Glue -->|Bytes| Rewriter[Streaming Rewriter (lol_html)]
    Rewriter -->|Target Bytes| Output[Output Stream]
```

## 2. Implementation Roadmap

### Phase 1: Core Engine & The Glue (Foundation Building)

Goal: Complete a Rust-only prototype that converts Markdown to HTML at high speed.

* Repository Setup
  * [ ] Set up repository
  * [ ] Create Rust workspace (`cargo new --lib`)
  * [ ] Initial CI/CD pipeline configuration (GitHub Actions)
* Core Parser Implementation
  * [ ] Introduce `markdown-rs`)
  * [ ] Verify basic Markdown parsing functionality
* The Glue (PipeAdapter) Implementation **【Top Priority】**
  * [ ] Implement an adapter that accepts `Iterator<Item=Event>` and streams it to `io::Write`
  * [ ] Benchmark stream connection to avoid buffering
* Streaming Rewriter Integration
  * [ ] Embed `lol_html`
  * [ ] Implement HTML tag rewriting (e.g., adding `lazy` attribute to `<img>` tags)
* NAPI Bindings
  * [ ] Set up `napi-rs`
  * [ ] Expose a basic function that accepts a string and returns a string from Node.js

### Phase 2: Astro Integration (Content Layer API)

Goal: Make the engine functional as an Astro v5 Loader and usable in real-world projects.

* Astro Loader Prototype
      * [ ] Implement a Loader readable from `src/content/config.ts`
      * [ ] Output data that conforms to the `RenderedContent` type
* MDX Support (Step 1: Hybrid Bridge)
      * [ ] Process to pass JSX blocks through as "raw text"
      * [ ] Verification of handover to the standard Astro pipeline
* MDX Support (Step 2: Native Compiler)
      * [ ] Introduce `swc_core`
      * [ ] Implement JSX code generation logic within Rust
      * [ ] Detect `client:*` directives and maintain hydration
* Performance Measurement
      * [ ] Compare build times for a 1000-page site (vs. Standard Astro)

### Phase 3: Universal Deployment (Nuxt & Next.js)

Goal: Abstract framework-specific differences and expand the ecosystem.

#### Nuxt (Content v3) Support

* [ ] Transformer Implementation
  * [ ] Support for MDC syntax (`::alert`) via `markdown-rs` extensions
  * [ ] Serialization process to Minimal Tree (JSON AST) for Nuxt
* [ ] Metadata Extraction
  * [ ] Output structured data for SQL storage

#### Next.js (App Router / Turbopack) Support

* [ ] Webpack Loader Creation
  * [ ] Implement `markflow-loader`
* [ ] RSC (React Server Components) Support
  * [ ] JSX generation for Server Components
* [ ] `unplugin` Integration
  * [ ] Unify build configuration (Vite/Webpack/Rspack)

### Phase 4: Ecosystem & Optimization (Future)

* [ ] WASM Plugin System: Mechanism to load user-defined logic via WASM.
* [ ] Rust-native Syntax Highlighting: Integrate `syntect` or `tree-sitter`.
* [ ] Turbopack Native Plugin: Support for Next.js's official Rust plugin system.

## 3. Technology Stack Selection

| Category | Technology/Crate | Purpose |
| :--- | :--- | :--- |
| **Language** | **Rust 2021** | Core Logic |
| **Parser** | **`markdown-rs`** | MDX/CommonMark Parsing (micromark compatible) |
| **Bridge** | **`napi-rs`** | High-speed communication with Node.js (V8) |
| **Rewriter** | **`lol_html`** | Streaming HTML Rewriting |
| **AST/Gen** | **`swc_core`** | JavaScript/JSX AST Construction and Code Generation |
| **Concurrency**| **`rayon`** | Parallel Processing (File reading, etc.) |
| **Highlight** | **`syntect`** | Build-time Syntax Highlighting |

## 4. Risks and Issue Management ⚠️

* MDX Complexity: The limit of accuracy for parsing nested JSX and dynamic JavaScript expressions.
* Turbopack Specification Changes: Potential follow-up costs due to the Next.js plugin API being undetermined.
* Ecosystem Compatibility: Speed of providing alternatives (WASM plugins) for the inability to use existing Remark plugins.
