# MF-130 WASM Streaming Adapter Parity

## 背景
WASM 版は `crates/wasm` で最低限のバインディングしか提供しておらず、Rust コアの最新機能（RewriteOptions, stats など）を公開していない。

## オーナー / 期限
- Owner: Miki (WASM)
- Reviewer: Kenji (Core)
- Target ship date: TBD（API 方針決定後 2 週間以内）

## 要求
- `crates/wasm` に `parse_with_stats` を追加し、`RewriteOptions` と同等のオプションを受け取れるようにする。
- `wasm-bindgen-test` で CI 可能な最小限のテストを追加する。
- npm 向けの README スニペット（`docs/specs/mf-130-wasm.md` で定義）を `docs/development/spec-driven-codex.md` から参照できるようリンクを張る。

## ブロッカー
- PipeAdapter の WASM サポート設計がまだ承認されていないため、`Backlog.md` の Status は `Blocked` のまま保持する。
- API 方針が固まるまで Codex に実装を依頼しないこと。

## 完了条件
1. コア機能の parity が実証されるベンチ or テストログがあること。
2. `Backlog.md` のステータスが `Done` に変更され、決定メモが残っていること。

## 受け入れテスト（API 決定後に有効）
1. `wasm-pack test --headless --chrome` で `parse_with_stats` の wasm-bindgen-test がパスする。
2. `npm pack crates/wasm/pkg` 後に README スニペットへ stats の使用例が自動的に差し込まれる。
3. `scripts/smoke-wasm.mjs`（後続タスクで追加）を実行すると、Rust コアと一致する統計値がログ出力される。
