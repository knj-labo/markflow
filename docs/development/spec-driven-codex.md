# Codexを活用した仕様駆動開発フロー

## 概要
「仕様駆動開発」に Codex を組み合わせ、`Backlog.md` でタスクを一元管理しながら、Codex に実装計画の立案とコード生成を任せる手順をまとめた実例です。要求仕様 → 計画 → 実装 → レビューの流れをすべてリポジトリ内で可視化し、履歴を残します。

## 事前準備
1. 仕様ドキュメント: `docs/` 以下に機能ごとの仕様（例: `docs/specs/mf-101-streaming.md`）を用意し、前提・制約・完了条件を記述します。
2. Backlog: ルート直下の `Backlog.md` に各タスク ID、仕様リンク、現在の状態、次に Codex が実行すべきステップを記録します。
3. ガイドライン遵守: 実装手順は [AGENTS.md](../../AGENTS.md) および `docs/development/guidelines*.md` に沿わせます。

## フロー
1. **仕様の確定**: 追加したい機能について仕様ファイルを作成し、完了条件と受け入れテストを明記します。
2. **Backlog 更新**: `Backlog.md` にタスク行を追加して「Status=Ready」に設定。仕様ファイルと関連ブランチをリンクします。
3. **Codex への依頼**: タスク ID と仕様へのパスを提示し、「Plan Tool を使って 2〜3 ステップの計画を作成した後、各ステップを順番に実装してほしい」と指示します。
4. **計画レビュー**: Codex が提示した計画を確認し、必要に応じてコメントや追加制約を返します。合意したら実装を進めてもらいます。
5. **実装 & テスト**: Codex に必要なコマンド（`cargo test --workspace`, `pnpm test`, `node scripts/smoke-napi.mjs ...` など）を実行させ、結果を要約してもらいます。
6. **Backlog 更新 & PR**: タスク完了後に `Backlog.md` のステータスを `Done` に変更し、PR 説明にタスク ID と仕様リンク、Codex が行った手動検証のログを貼ります。

## Backlog バリデーション
- Pull Request 前に `node scripts/check-backlog.mjs` を実行し、各タスク行の Spec パスとヘッダーが一致することを確認します。
- `.github/workflows/ci.yml` の `Validate backlog specs` ステップ（Node 20 上で実行）が自動的に同じコマンドを呼び、Backlog への手動編集漏れを防ぎます。
- 失敗時には足りない spec ファイルや見出しの不一致が表示されるため、修正コミットで揃えてから再度実行してください。

## Codex へのプロンプト例
```
タスク: MF-101 Streaming Rewriter Diagnostics (Backlog.md 参照)
仕様: docs/specs/mf-101-streaming.md
要求:
1. Plan Tool で 3 ステップ以内の計画を提案
2. 各ステップを順番に実行し、変更ファイルとテスト結果を報告
3. ガイドライン違反があれば自己修正
```

## 運用のコツ
- Backlog の「Next」欄に、Codex が最初にすべきアクション（例: "Read docs/specs/..."）を書いておくと迷子になりません。
- 仕様が変わったら必ず Backlog と仕様ファイルを同じコミットで更新し、Codex に再同期させます。
- Codex が出力した計画はそのままドキュメントに貼り付けておくと、後続の振り返り資料として再利用できます。
