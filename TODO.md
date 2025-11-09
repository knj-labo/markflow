# TODO

現在の実装を安定的に保ちながら、段階的なRSMDロードマップを追跡しています。

## 近期のPR

1. PR0 – ビルド修正 + スモークテスト
   - `packages/core/src/slugify.rs`内の`use crate::HashSet;`を`use std::collections::HashSet;`に置換する。
   - `cargo build && cargo test`が正常に動作するよう、少なくとも1つの基本的な単体テストを追加する。

2. PR1 – pulldown-cmark配線
   - `pulldown-cmark`を使用して実際の`<p>`/`<h1>` HTMLを出力する。
   - `render('# A')`が`<h1>…</h1>`と有効な`RenderResult`を返すことを確認する。

3. PR2 – 見出し収集（スラグなし）
   - パーサーイベントを走査してレベル1見出し`{ depth, text }`を収集する。
   - テスト: `'# A\n## B'`が`headings`内で`A`のみを生成することを確認する。

4. PR3 – ASCII スラグ化
   - 衝突処理付きASCIIのみのスラグ生成を実装する（`a`, `a-2`, `a-3`, …）。
   - レベル1見出しにスラグを添付する。

5. PR4 – Unicode スラグ化
   - CJK対応ルール（空白除去、句読点削除など）でスラグ化を拡張する。
   - 日本語/混合見出しのテストを追加する。

6. PR5 – 見出し範囲拡張
   - h1～h3を収集し、スラグを割り当て、生成されたHTMLに`id="slug"`を挿入する。

7. PR6 – wasm-bindgen 最小限エクスポート
   - `wasm32-unknown-unknown`をターゲットとする際に既存のクレートに対して`render_wasm`を公開する。
   - `wasm-pack build --target bundler`を成功させる。

8. PR7 – Node スモークテスト
   - WASMバンドルを読み込み`render('# A')`をアサートするVitestを含む`examples/node/`を追加する。
   - `pnpm test:node`をCIに組み込む。

9. PR8 – ブラウザスモークテスト
   - `examples/web/`（例：Vite）とWASMレンダーパスを実行するPlaywrightテストを追加する。

10. PR9 – README現実チェック
    - READMEをRSMD MVPに焦点を当てるよう更新し、ハイブリッド計画を「将来の作業」に移動し、クイックスタートガイドを追加する。

11. PR10 – 最小限CLI（オプション）
    - 見出し出力用の`--json`付き`stdin`→`stdout` CLIを提供し、READMEに文書化する。

## バックログ

- h4～h6見出しとTOCユーティリティのサポート。
- HTML対応範囲（リスト、コードブロック、リンク）を小さな増分で拡張。
- リグレッション用のスナップショット/プロパティテストを追加。
- エラータイプとpublic ABIを安定化。
- WASM vs. ネイティブバイナリのパッケージング戦略を評価。
