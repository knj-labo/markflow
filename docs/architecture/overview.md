# Architecture Overview

> This document provides a high-level overview of Markflow's architecture.

## System Architecture

Markflow is a streaming Markdown/MDX engine built with Rust, designed for high-performance processing in web frameworks.

### Core Components

1. **Core Engine** (`crates/core`)
   - Markdown-rs based parser
   - Pull-to-push pipeline adapter (PipeAdapter)
   - Streaming rewriter hooks
   - No AST materialization for memory efficiency

2. **NAPI Bindings** (`crates/napi`)
   - Node.js interface via NAPI-RS
   - Zero-copy streaming to JavaScript
   - Benchmark comparisons with remark

3. **WASM Bindings** (`crates/wasm`)
   - WebAssembly interface
   - Browser and edge runtime support

## Data Flow

```
Markdown Input
     ↓
  Parser (markdown-rs)
     ↓ (pull-based events)
  PipeAdapter (pull → push)
     ↓ (streaming)
  lol_html Rewriter
     ↓
  HTML Output
```

## Design Principles

- **Streaming-first**: Process content without full AST materialization
- **Memory efficiency**: Direct streaming without intermediate buffers
- **Performance**: Target sub-100ms processing for 10MB files
- **Extensibility**: Plugin-based rewriter hooks

## References

- Full implementation roadmap: [ROADMAP.md](../../ROADMAP.md)
- Development guidelines: [guidelines.md](../development/guidelines.md)
