# Repository Guidelines

## Project Structure & Workspace
Phase 1 assumes a Cargo workspace with `crates/core`, `crates/napi`, and `crates/wasm`. Core owns the markdown-rs pipeline plus PipeAdapter, while bindings only expose interfaces. Place benchmarks and profiling data in `benchmarks/`, large Markdown fixtures in `fixtures/markdown/`, and helper scripts in `scripts/` (chmod +x). Keep PRDs or diagrams in `docs/` so architecture conversations stay versioned.

## Build, Test, and Development Commands
Use `cargo fmt` and `cargo clippy --workspace --all-targets` before pushing. Validate the Rust engine via `cargo test --workspace` and `cargo bench -p markflow-core --features bench` to watch memory when processing 10 MB samples. The Node bridge flow is `pnpm install`, `pnpm run build:napi`, then `node scripts/smoke-napi.mjs samples/large.md` to compare against remark using `console.time`.

## Parser & Glue Expectations
When extending Task 1.2, keep parser changes as event iterators—no AST materialization. PipeAdapter (Task 1.3) must implement `std::io::Write` and stream directly into `lol_html` without temporary buffers; document any allocation hotspots in code comments. Rewriter hooks (Task 1.4) should prove value with concrete examples such as auto-injecting `loading="lazy"` on `<img>`.

## Coding Style & Naming
Follow Rust 2021 defaults: 4-space indent, snake_case files, CamelCase types. Adapter implementations live in `*_adapter.rs`; rewriters in `*_rewriter.rs`. Enable `#![deny(missing_docs)]` per crate and describe how each struct participates in the pull→push pipeline. Prefer ESM with named exports on the JS side and keep filenames lowercase-hyphenated.

## Testing & Benchmarks
Unit tests sit beside code (`mod tests`) and draw inputs from `fixtures/`. For streaming/regression coverage, add Criterion benches rather than huge snapshots, and record peak RSS in `benchmarks/results.md`. Node bindings should gain AVA/Vitest specs under `crates/napi/__tests__/` named `<feature>.spec.ts`. Aim for >85 % coverage inside `core::parser` and add a benchmark for each new rewrite rule.

## Commit & PR Workflow
Use Conventional Commits (`feat: parser events`, `perf: glue rss drop`). Every PR must include: summary, affected tasks from TODO.md, perf evidence (bench result or memory graph), and instructions for verifying `node scripts/smoke-napi.mjs`. Reference linked issues and attach screenshots for CLI output when relevant. Reject PRs that skip lint/test sections or introduce secrets; rely on `.env.local` with `dotenvy`.
