# リポジトリガイドライン

## プロジェクト構成とワークスペース
Phase 1 では `crates/core`、`crates/napi`、`crates/wasm` を含む Cargo ワークスペースを前提とします。Core は markdown-rs パイプラインと PipeAdapter を持ち、バインディング側はインターフェースだけを公開します。ベンチやプロファイル結果は `benchmarks/`、大きな Markdown フィクスチャは `fixtures/markdown/`、補助スクリプトは `scripts/`（実行権を付与）に配置します。PRD やダイアグラムは `docs/` に置いて議論内容をバージョン管理します。

## ビルド・テスト・開発コマンド
push 前に `cargo fmt` と `cargo clippy --workspace --all-targets` を実行します。Rust エンジンは `cargo test --workspace` と `cargo bench -p markflow-core --features bench` で検証し、10 MB サンプル処理時のメモリを確認します。Node ブリッジは `pnpm install`、`pnpm run build:napi`、`node scripts/smoke-napi.mjs samples/large.md` を順に実行し、`console.time` で remark と比較します。

## パーサーとグルーの要件
Task 1.2 を拡張する際は、パーサー変更をイベントイテレータとして保ち AST を生成しないでください。PipeAdapter（Task 1.3）は `std::io::Write` を実装し、一時バッファを作らず `lol_html` へストリームします。割り当てホットスポットがあればコードコメントで説明します。Rewriter フック（Task 1.4）は `<img>` に `loading="lazy"` を自動付与するなど、価値を示す具体例を用意します。

## コーディングスタイルと命名
Rust 2021 のデフォルトに従い、インデントは 4 スペース、ファイル名は snake_case、型名は CamelCase とします。アダプタの実装は `*_adapter.rs`、リライタは `*_rewriter.rs` に配置します。各クレートで `#![deny(missing_docs)]` を有効にし、pull→push パイプライン内での構造体の役割を説明します。JS 側は ESM の名前付きエクスポートを優先し、ファイル名は小文字＋ハイフンに揃えます。

## テストとベンチマーク
ユニットテストはコード付近（`mod tests`）に置き、入力は `fixtures/` から取得します。ストリーミング／リグレッションのカバレッジには巨大スナップショットではなく Criterion ベンチを使い、ピーク RSS を `benchmarks/results.md` に記録します。Node バインディングは `crates/napi/__tests__/` 配下に AVA/Vitest の `<feature>.spec.ts` を追加します。`core::parser` では 85% 以上のカバレッジを目指し、リライトルールを追加するたびにベンチを用意してください。

## 実装ロードマップ（Phase 1 抜粋）
- Core Parser Implementation
  - [ ] `markdown-rs` の導入
  - [ ] Markdown 基本機能の検証

## コミットと PR ワークフロー
Conventional Commits（例: `feat: parser events`, `perf: glue rss drop`）を使用します。全ての PR に summary、TODO.md で影響を受けたタスク、パフォーマンスエビデンス（ベンチ結果またはメモリグラフ）、`node scripts/smoke-napi.mjs` の検証手順を含めます。関連 Issue を参照し、必要に応じて CLI 出力のスクリーンショットを添付します。lint/test を省略する PR やシークレットを含む PR は拒否し、`.env.local` と `dotenvy` を利用してください。
