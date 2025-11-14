# TODO - Reflow Roadmap

🌊 **Reflow** - Blazingly fast Markdown & MDX compiler powered by Rust

現在のプロジェクトを Reflow としてリブランディングし、段階的に Markdown 専用エンジンから真のハイブリッド MDX コンパイラへと進化させていきます。

---

## 🎯 プロジェクトビジョン

**短期目標（1-2ヶ月）:**
- ✅ CJK/Unicode 対応の高速 Markdown レンダラーとして完成
- ✅ `@reflow/markdown` v0.1.0 公開

**中期目標（3-4ヶ月）:**
- 🚧 mdxjs-rs を統合した MDX サポート
- 🚧 Astro Content Collections 統合
- 🚧 `@reflow/core`, `@reflow/mdx`, `@reflow/astro` 公開

**長期目標（6ヶ月+）:**
- 📋 他フレームワーク対応（Next.js, Vite, etc.）
- 📋 プラグインエコシステム
- 📋 v1.0.0 安定版リリース

---

## 📦 パッケージ構成

```
@reflow/core        # ハイブリッドルーター（Markdown/MDX自動判定）
@reflow/markdown    # Markdown専用エンジン（Rust/WASM - pulldown-cmark）
@reflow/mdx         # MDX専用エンジン（Rust/WASM - mdxjs-rs wrapper）
@reflow/astro       # Astro integration
```

---

## Phase 0: Rebranding & Foundation（Week 1-2）

### 目標: Reflow としての再出発

#### ✅ 完了済み
- [x] プロジェクト名決定: **Reflow**
- [x] npm パッケージ名確認: `@reflow/*` 利用可能
- [x] TODO.md ロードマップ作成

#### 🚧 進行中
- [ ] **GitHub リポジトリ名変更**
  - `mdx-hybrid` → `reflow`
  - リモート URL 更新

- [ ] **package.json 更新**
  - すべてのパッケージ名を `@reflow/*` に変更
  - 依存関係の整理

- [ ] **Cargo.toml 更新**
  - crate 名を `reflow-*` に変更
  - workspace 構成の見直し

- [ ] **ディレクトリ構造整理**
  ```
  packages/
    core/        # reflow-core (Rust - Markdown engine)
    mdx/         # reflow-mdx (Rust - MDX engine, 新規)
    reflow/      # @reflow/core (TypeScript - hybrid router, 新規)
    markdown/    # @reflow/markdown (TypeScript wrapper, 新規)
    mdx/         # @reflow/mdx (TypeScript wrapper, 新規)
    astro/       # @reflow/astro (Astro integration, 新規)
  ```

- [ ] **ドキュメント更新**
  - README.md リブランディング
  - DESIGN.md 作成（RSMD_DESIGN.md から移行）
  - ロゴ・ブランディング素材準備

**成果物:**
- リブランディング完了
- プロジェクト構造整備
- 新しい README.md

---

## Phase 1: Markdown Core 完成（Week 3-6）

### 目標: `@reflow/markdown` v0.1.0 リリース

#### PR1 – 見出し範囲拡張（H1-H6）
- [ ] H2-H6 見出しの抽出対応
- [ ] 見出しレベル情報の保持
- [ ] 階層構造の正確な表現
- [ ] テストケース追加（各レベル）

#### PR2 – シングルパス最適化
- [ ] HTML 生成と見出し収集の同時実行
- [ ] 2パス構成からの脱却
- [ ] パフォーマンスベンチマーク
  - 目標: remark-parse の 5-10x 高速化
  - 小サイズ (1KB): <1ms
  - 中サイズ (10KB): <5ms
  - 大サイズ (100KB): <50ms

#### PR3 – WASM 統合完了
- [ ] モック実装を削除
- [ ] 実際の wasm-pack ビルド統合
- [ ] TypeScript からの WASM 呼び出し確認
- [ ] エラーハンドリング改善

#### PR4 – テスト & ドキュメント
- [ ] ユニットテスト拡充（カバレッジ 90%+）
- [ ] 統合テスト追加
- [ ] パフォーマンステスト
- [ ] API ドキュメント作成

#### PR5 – npm 公開準備
- [ ] ビルドパイプライン整備
- [ ] package.json メタデータ完成
- [ ] CHANGELOG.md 作成
- [ ] LICENSE 確認

**成果物:**
- [x] `@reflow/markdown` v0.1.0 公開
- [x] ベンチマーク結果公開
- [x] 完全なドキュメント

**マイルストーン:** Month 1 完了

---

## Phase 2: MDX Integration（Week 7-12）

### 目標: `@reflow/mdx` と `@reflow/core` v0.1.0 リリース

#### PR6 – MDX エンジン統合
- [ ] `packages/mdx/` 作成（Rust）
- [ ] mdxjs-rs 依存関係追加
  ```toml
  [dependencies]
  mdxjs = "1.0"
  ```
- [ ] WASM バインディング実装
- [ ] 基本的な MDX コンパイル動作確認

#### PR7 – TypeScript MDX ラッパー
- [ ] `@reflow/mdx` パッケージ作成
- [ ] mdxjs-rs WASM との統合
- [ ] TypeScript 型定義
- [ ] エラーハンドリング

#### PR8 – ハイブリッドルーター実装
- [ ] `@reflow/core` パッケージ作成
- [ ] フォーマット自動判定ロジック
  - ファイル拡張子（.md / .mdx）
  - 内容の解析（JSX、import/export の検出）
  - オプションによる明示的指定
- [ ] Markdown/MDX エンジンへのルーティング
- [ ] 統合 API デザイン
  ```typescript
  import {compile} from '@reflow/core';

  // 自動判定
  const result = await compile(source);

  // 明示的指定
  const result = await compile(source, {format: 'mdx'});
  ```

#### PR9 – テスト & ドキュメント
- [ ] MDX 機能テスト
  - JSX コンポーネント
  - import/export
  - Expression 構文
- [ ] ハイブリッドルーターテスト
- [ ] 使用例・チュートリアル作成

**成果物:**
- [ ] `@reflow/mdx` v0.1.0 公開
- [ ] `@reflow/core` v0.1.0 公開
- [ ] MDX サポート完了

**マイルストーン:** Month 2 完了

---

## Phase 3: Astro Integration（Week 13-16）

### 目標: `@reflow/astro` v0.1.0 リリース & 実用化

#### PR10 – Astro Integration パッケージ
- [ ] `@reflow/astro` パッケージ作成
- [ ] Astro Integration API 実装
  ```typescript
  import reflow from '@reflow/astro';

  export default {
    integrations: [reflow()],
  };
  ```
- [ ] Content Collections サポート
- [ ] カスタムローダー実装

#### PR11 – 実例プロジェクト
- [ ] `examples/astro-blog/` 作成
  - .md ファイル（Reflow Markdown engine）
  - .mdx ファイル（Reflow MDX engine）
  - 日本語コンテンツでの動作確認
- [ ] `examples/astro-docs/` 作成
  - 技術ドキュメントサイト
  - TOC 生成
  - 多言語対応

#### PR12 – ドキュメントサイト
- [ ] Astro + Reflow で公式サイト構築
- [ ] 日英両対応
- [ ] パフォーマンスベンチマーク掲載
- [ ] インタラクティブデモ
- [ ] GitHub Pages / Vercel デプロイ

#### PR13 – Astro コミュニティ対応
- [ ] Astro Discord での紹介
- [ ] Integration ディレクトリへの登録申請
- [ ] ブログ記事・チュートリアル作成
- [ ] フィードバック収集

**成果物:**
- [ ] `@reflow/astro` v0.1.0 公開
- [ ] 動作する実例サイト 2件以上
- [ ] 公式ドキュメントサイト
- [ ] Astro コミュニティでの認知

**マイルストーン:** Month 3 完了

---

## Phase 4: Optimization & Ecosystem（Week 17+）

### 目標: v1.0.0 安定版リリース & エコシステム拡大

#### パフォーマンス最適化
- [ ] キャッシング戦略実装
- [ ] 増分コンパイル
- [ ] 並列処理最適化
- [ ] メモリ使用量最適化
- [ ] バンドルサイズ削減

#### 他フレームワーク対応
- [ ] `@reflow/next` - Next.js integration
- [ ] `@reflow/vite` - Vite plugin
- [ ] `@reflow/rollup` - Rollup plugin
- [ ] `@reflow/webpack` - Webpack loader

#### プラグインシステム（検討）
- [ ] Rust プラグイン API 設計
- [ ] または JavaScript プラグインブリッジ
- [ ] remark/rehype プラグイン互換性調査

#### コミュニティ & エコシステム
- [ ] GitHub Discussions 活性化
- [ ] Issue/PR テンプレート整備
- [ ] コントリビューションガイド
- [ ] 技術記事・ブログ執筆
- [ ] カンファレンス発表（Rust/JS）

#### 安定化 & v1.0.0
- [ ] API の最終決定
- [ ] 破壊的変更の整理
- [ ] セマンティックバージョニング遵守
- [ ] 長期サポート計画
- [ ] セキュリティポリシー

**成果物:**
- [ ] 他フレームワーク統合 2件以上
- [ ] コミュニティ貢献者 5名以上
- [ ] GitHub Stars 100+
- [ ] npm 週間 DL 500+
- [ ] v1.0.0 リリース

**マイルストーン:** Month 4+ 継続的改善

---

## バックログ・将来の機能

### 高優先度
- [ ] H4-H6 見出しと TOC ユーティリティ
- [ ] シンタックスハイライト統合
- [ ] 画像最適化サポート
- [ ] フロントマター拡張

### 中優先度
- [ ] リグレッションテスト（スナップショット）
- [ ] プロパティベーステスト
- [ ] CI/CD パイプライン完全自動化
- [ ] パフォーマンス回帰検出

### 低優先度 / 検討中
- [ ] setext 見出しサポート
- [ ] カスタム Markdown 拡張
- [ ] WASM vs ネイティブバイナリ戦略
- [ ] CommonMark 完全準拠レベルの明確化

---

## 📊 成功指標（KPI）

### 技術指標
- [ ] Markdown 処理が remark より **5-10x 高速**
- [ ] CJK 対応率 **100%** (Unicode 16.0 準拠)
- [ ] テストカバレッジ **90%以上**
- [ ] ドキュメント完備率 **100%**

### コミュニティ指標
- [ ] GitHub Stars **100+**
- [ ] npm 週間ダウンロード **500+**
- [ ] 技術記事・発表 **3件以上**
- [ ] コントリビューター **5名以上**

### 実用化指標
- [ ] プロダクション利用例 **3件以上**
- [ ] Astro Integration Directory 掲載
- [ ] Issue/PR 対応率 **80%以上**
- [ ] ユーザーフィードバック **10件以上**

---

## 🔗 リソース

- **GitHub**: https://github.com/knj-labo/reflow (予定)
- **npm**: https://www.npmjs.com/org/reflow
- **Docs**: (Phase 3 で構築予定)
- **Discord**: Astro Discord #integrations チャンネル

---

## 📝 メモ

### プロジェクト履歴
- 2024-11: `mdx-hybrid` プロジェクトとして開始
- 2024-11: CJK/Unicode slugification 完成 (PR1)
- 2025-11: **Reflow** としてリブランディング
- 2025-Q1: Markdown エンジン完成目標
- 2025-Q2: MDX & Astro 統合目標

### 命名の由来
**Reflow** = データが流れるように処理される様子
- remark/rehype との親和性
- 「速く流れる」= Rust のパフォーマンス
- 「リフロー」= 再構成・変換のプロセス

---

最終更新: 2025-11-14
