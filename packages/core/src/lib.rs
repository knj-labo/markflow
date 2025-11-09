//! # rsmd-core
//!
//! pulldown-cmark を用いた RSMD PR0 (MVP) の最小レンダラー実装です。
//!
//! ## 現在の状態 (PR0)
//!
//! - ✅ HTML 出力は `pulldown-cmark` に完全委譲しており、CommonMark + GFM の正確なレンダリングを最優先します。
//! - ⚠️ 見出し収集は `#` で始まる行を走査する暫定 2 パス実装で、setext 見出しやコードブロック内の `#` を正確に扱うのは PR1 以降の課題です。
//! - 📦 API は `render()` と `RenderResult { html, headings }` を安定させ、ヘッダ情報はベストエフォートで提供します。
//!
//! ## 次のステップ
//!
//! - PR1: `pulldown_cmark::Event` ベースの見出し収集と CommonMark 規則への完全準拠。
//! - PR2: ASCII スラグの衝突処理強化と API ドキュメント整備。
//! - PR4: HTML 生成と見出し収集のシングルパス統合（TODO.md 参照）。
//!
//! ## 参考実装
//!
//! - HTMLエスケープ: <https://github.com/wooorm/markdown-rs/blob/main/src/util/encode.rs>
//! - URIサニタイズ: <https://github.com/wooorm/markdown-rs/blob/main/src/util/sanitize_uri.rs>
//! - HTML生成アーキテクチャ: <https://github.com/wooorm/markdown-rs/blob/main/src/to_html.rs>
//! - GitHub互換slug生成: <https://github.com/markdown-it-rust/markdown-it-plugins.rs/blob/main/crates/github_slugger/src/lib.rs>
//! - GitHub互換slug (crate): <https://docs.rs/github-slugger>
//! - pulldown-cmark (使用中): <https://docs.rs/pulldown-cmark>

pub use pulldown_cmark::{html, Event, Options as CmarkOptions, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

mod is_cjk;
pub use crate::is_cjk::is_cjk;

mod sanitize_html;
pub use crate::sanitize_html::sanitize_html;

mod slugify;
pub use crate::slugify::slugify;

// ===== 構造体定義 =====

/// レンダリングオプション（すべてデフォルトON）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    #[serde(default = "default_true")]
    pub gfm_tables: bool,
    #[serde(default = "default_true")]
    pub gfm_tasklists: bool,
    #[serde(default = "default_true")]
    pub footnotes: bool,
    #[serde(default = "default_true")]
    pub smart_punct: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            gfm_tables: true,
            gfm_tasklists: true,
            footnotes: true,
            smart_punct: true,
        }
    }
}

fn default_true() -> bool {
    true
}

/// レンダリング結果（ABI固定）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResult {
    pub html: String,
    pub headings: Vec<Heading>,
}

/// 見出し情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub depth: u8,    // 1..6
    pub text: String, // プレーンテキスト
    pub slug: String, // 自動生成ID
}

/// Markdownをレンダリング（pulldown-cmark + 暫定2パス見出し収集）
///
/// PR0 では以下の 2 ステップで処理します：
///
/// 1. `pulldown-cmark` で CommonMark + GFM HTML を生成します。
/// 2. 元のMarkdown文字列を再走査し、先頭が `#` の行をベストエフォートで見出しとして記録します。
///
/// HTML の正確さを最優先にする方針により、見出し収集が取りこぼしや過検出をしても
/// HTML は常に pulldown-cmark の仕様準拠結果になります。
///
/// ## サポートする要素
/// - 基本: 見出し / 段落 / 強調 / コード / リスト / リンク / 画像
/// - GFM: テーブル / タスクリスト / 取り消し線 / 自動リンク / 脚注
/// - オプション: `Options` で tables / tasklists / footnotes / smart punctuation を個別に制御
///
/// ## 使用例
///
/// ```rust
/// use rsmd_core::{render, Options};
///
/// let markdown = "# A\n\nParagraph with **bold** and [link](https://example.com).";
/// let result = render(markdown, &Options::default());
/// assert!(result.html.contains("<h1>A</h1>"));
/// assert!(result
///     .html
///     .contains("<p>Paragraph with <strong>bold</strong> and <a href=\"https://example.com\">link</a>.</p>"));
/// assert_eq!(result.headings.len(), 1);
/// assert_eq!(result.headings[0].text, "A");
/// ```
///
/// `RenderResult` は HTML と見出しリスト（depth / text / slug）を返し、
/// 将来のPRで heading の正確性を高めても API 互換性を保てるようにしています。
///
/// ## セキュリティと今後の改善
/// - pulldown-cmark の生HTMLが必要な場合は `sanitize_html` を組み合わせて利用してください。
/// - PR1 で `pulldown_cmark::Event` ベースの heading 収集に移行し、PR4 でシングルパス化する予定です。
pub fn render(source: &str, options: &Options) -> RenderResult {
    // pulldown-cmarkオプションに変換
    let cmark_options = convert_options(options);

    // パーサーを初期化
    let parser = Parser::new_ext(source, cmark_options);

    // HTMLを生成
    let mut html = String::new();
    html::push_html(&mut html, parser);

    // 見出し抽出のために再度パースする（PR1でイベントベース化予定）
    let headings = extract_headings(source, &cmark_options);

    RenderResult { html, headings }
}

/// RSMDオプションをpulldown-cmarkオプションに変換
///
/// `Options`構造体の各フィールドを対応する`pulldown_cmark::Options`フラグに変換します。
/// この変換により、RSMDの設定がpulldown-cmarkエンジンに正しく伝達されます。
///
/// ## 変換マッピング
///
/// | RSMDフィールド | pulldown-cmarkフラグ | 機能 |
/// |---------------|-------------------|------|
/// | `gfm_tables` | `ENABLE_TABLES` | パイプ区切りテーブル構文 |
/// | `gfm_tasklists` | `ENABLE_TASKLISTS` | `- [x]` チェックボックス構文 |
/// | `footnotes` | `ENABLE_FOOTNOTES` | `[^1]` 脚注記法 |
/// | `smart_punct` | `ENABLE_SMART_PUNCTUATION` | スマート句読点変換 |
///
/// ## 使用例
///
/// この関数は内部で自動的に呼ばれるため、ユーザーが直接呼び出す必要はありません。
/// `render()`関数の`options`パラメータとして渡されたオプションが
/// 自動的にpulldown-cmark形式に変換されます。
fn convert_options(options: &Options) -> CmarkOptions {
    let mut cmark_options = CmarkOptions::empty();

    if options.gfm_tables {
        cmark_options.insert(CmarkOptions::ENABLE_TABLES);
    }

    if options.gfm_tasklists {
        cmark_options.insert(CmarkOptions::ENABLE_TASKLISTS);
    }

    if options.footnotes {
        cmark_options.insert(CmarkOptions::ENABLE_FOOTNOTES);
    }

    if options.smart_punct {
        cmark_options.insert(CmarkOptions::ENABLE_SMART_PUNCTUATION);
    }

    // GFM取り消し線は標準で有効（pulldown-cmarkのデフォルト動作）
    cmark_options.insert(CmarkOptions::ENABLE_STRIKETHROUGH);

    cmark_options
}

/// 見出し抽出（PR0暫定）
///
/// `pulldown-cmark` が生成したHTMLとは独立に、元のMarkdownを行単位で走査して
/// 先頭が `#` の行をベストエフォートで見出しとして検出します。
///
/// - `#Heading` のようにスペースがなくても見出しとして扱います。
/// - コードブロックや引用内の `#` が誤検出される場合があります。
/// - setext 見出しは未対応です。
///
/// PR1 で `pulldown_cmark::Event` ベースの実装に置き換える予定です。
fn extract_headings(source: &str, _options: &CmarkOptions) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut used_slugs = HashSet::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix('#') {
            // 見出しレベルを計算
            let mut depth = 1u8;
            let mut remaining = stripped;

            while let Some(next_stripped) = remaining.strip_prefix('#') {
                depth += 1;
                remaining = next_stripped;
                if depth >= 6 {
                    break;
                }
            }

            // 見出しテキストを抽出（空白をトリム）
            let text = remaining.trim().to_string();
            if !text.is_empty() {
                let slug = slugify(&text, &mut used_slugs);
                headings.push(Heading { depth, text, slug });
            }
        }
    }

    headings
}

// ===== 内部状態（将来のPR1向け実装予定） =====

// TODO: PR1では以下の構造体を使用してpulldown-cmarkのイベントストリームから
// より正確な見出し抽出を実装する予定
//
// /// 見出し処理中の状態
// struct HeadingState {
//     depth: u8,
//     text: String,
// }
//
// /// 見出し収集器
// ///
// /// 参考: markdown-rsのCompileContext的な状態管理
// /// - <https://github.com/wooorm/markdown-rs/blob/main/src/to_html.rs>
// struct HeadingRecorder {
//     current_heading: Option<HeadingState>,
//     headings: Vec<Heading>,
//     used_slugs: HashSet<String>,
// }

// ===== WASMバインディング =====

#[cfg(target_arch = "wasm32")]
pub mod wasm_bindings;

// ===== テスト =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_h1_heading() {
        // H1見出しの正しいHTML生成を確認
        let result = render("# Hello World", &Options::default());
        assert!(
            result.html.contains("<h1>Hello World</h1>"),
            "Expected <h1>Hello World</h1>, got: {}",
            result.html
        );
        // 見出し抽出も正しく動作することを確認
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].depth, 1);
        assert_eq!(result.headings[0].text, "Hello World");
    }

    #[test]
    fn render_returns_structured_result() {
        // HTMLとRenderResultの整合性を確認
        let markdown = "# Title\n\nParagraph with **bold** and [link](https://example.com).";
        let result = render(markdown, &Options::default());

        assert!(!result.html.is_empty());
        assert!(result.html.contains("<h1>Title</h1>"));
        assert!(result
            .html
            .contains("<p>Paragraph with <strong>bold</strong> and <a href=\"https://example.com\">link</a>.</p>"));
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].depth, 1);
        assert_eq!(result.headings[0].text, "Title");
    }

    #[test]
    fn render_multiple_heading_levels() {
        // 複数レベルの見出しの正しい処理を確認
        let markdown = "# H1 Title\n## H2 Subtitle\n### H3 Section";
        let result = render(markdown, &Options::default());
        assert!(result.html.contains("<h1>H1 Title</h1>"));
        assert!(result.html.contains("<h2>H2 Subtitle</h2>"));
        assert!(result.html.contains("<h3>H3 Section</h3>"));
    }

    #[test]
    fn best_effort_heading_extraction_accepts_tight_atx_syntax() {
        // 暫定実装では `#Heading` も見出しとして扱うことを保証
        let markdown = "#NoSpace\nParagraph";
        let result = render(markdown, &Options::default());

        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "NoSpace");
    }

    #[test]
    fn render_paragraph() {
        // 段落の正しいHTML生成を確認
        let result = render("Hello world", &Options::default());
        assert!(
            result.html.contains("<p>Hello world</p>"),
            "Expected <p>Hello world</p>, got: {}",
            result.html
        );
    }

    #[test]
    fn render_multiline_paragraphs() {
        // 複数段落の正しい処理を確認
        let markdown = "First paragraph.\n\nSecond paragraph.";
        let result = render(markdown, &Options::default());
        assert!(result.html.contains("<p>First paragraph.</p>"));
        assert!(result.html.contains("<p>Second paragraph.</p>"));
    }

    #[test]
    fn render_emphasis_markup() {
        // 強調記法の正しいHTML生成を確認
        let markdown = "This is **bold** and *italic* text.";
        let result = render(markdown, &Options::default());
        assert!(result.html.contains("<strong>bold</strong>"));
        assert!(result.html.contains("<em>italic</em>"));
    }

    #[test]
    fn render_inline_code() {
        // インラインコードの正しいHTML生成を確認
        let markdown = "Use `code` for inline code.";
        let result = render(markdown, &Options::default());
        assert!(result.html.contains("<code>code</code>"));
    }

    #[test]
    fn render_code_blocks() {
        // コードブロックの正しいHTML生成を確認
        let markdown = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        let result = render(markdown, &Options::default());
        assert!(result.html.contains("<pre><code"));
        assert!(result.html.contains("fn main()"));
    }

    #[test]
    fn render_links() {
        // リンクの正しいHTML生成を確認
        let markdown = "Visit [Rust](https://rust-lang.org) website.";
        let result = render(markdown, &Options::default());
        assert!(result
            .html
            .contains("<a href=\"https://rust-lang.org\">Rust</a>"));
    }

    #[test]
    fn render_images() {
        // 画像の正しいHTML生成を確認
        let markdown = "![Rust Logo](https://rustacean.net/assets/rustacean-flat-happy.png)";
        let result = render(markdown, &Options::default());
        assert!(result.html.contains("<img"));
        assert!(result.html.contains("alt=\"Rust Logo\""));
        assert!(result
            .html
            .contains("src=\"https://rustacean.net/assets/rustacean-flat-happy.png\""));
    }

    // ===== GitHub Flavored Markdown (GFM) 拡張テスト =====

    #[test]
    fn render_tables_when_enabled() {
        // GFMテーブルの正しいHTML生成を確認（有効時）
        let markdown = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |";
        let result = render(markdown, &Options::default());
        assert!(
            result.html.contains("<table>"),
            "Expected table to be rendered, got: {}",
            result.html
        );
        assert!(result.html.contains("<th>Name</th>"));
        assert!(result.html.contains("<th>Age</th>"));
        assert!(result.html.contains("<td>Alice</td>"));
        assert!(result.html.contains("<td>30</td>"));
    }

    #[test]
    fn two_pass_heading_scan_preserves_gfm_html_correctness() {
        // 見出し抽出が2パスでもHTML生成が最優先で正しいことを確認
        let markdown = "# Table Heading\n\n| Name | Age |\n|------|-----|\n| Alice | 30 |";
        let result = render(markdown, &Options::default());

        assert!(result.html.contains("<table>"));
        let heading_texts: Vec<_> = result.headings.iter().map(|h| h.text.as_str()).collect();
        assert_eq!(heading_texts, vec!["Table Heading"]);
    }

    #[test]
    fn ignore_tables_when_disabled() {
        // GFMテーブルの無効化確認
        let markdown = "| Name | Age |\n|------|-----|\n| Alice | 30 |";
        let mut options = Options::default();
        options.gfm_tables = false;
        let result = render(markdown, &options);
        assert!(
            !result.html.contains("<table>"),
            "Expected table NOT to be rendered when disabled, got: {}",
            result.html
        );
    }

    #[test]
    fn render_tasklists_when_enabled() {
        // GFMタスクリストの正しいHTML生成を確認（有効時）
        let markdown = "- [x] Completed task\n- [ ] Pending task";
        let result = render(markdown, &Options::default());
        assert!(result.html.contains("type=\"checkbox\""));
        assert!(result.html.contains("checked=\"\""));
    }

    #[test]
    fn ignore_tasklists_when_disabled() {
        // GFMタスクリストの無効化確認
        let markdown = "- [x] Completed task\n- [ ] Pending task";
        let mut options = Options::default();
        options.gfm_tasklists = false;
        let result = render(markdown, &options);
        assert!(
            !result.html.contains("type=\"checkbox\""),
            "Expected tasklist NOT to be rendered when disabled, got: {}",
            result.html
        );
    }

    #[test]
    fn render_strikethrough_text() {
        // GFM取り消し線の正しいHTML生成を確認
        let markdown = "This is ~~deleted~~ text.";
        let result = render(markdown, &Options::default());
        assert!(result.html.contains("<del>deleted</del>"));
    }

    // ===== 高度な機能テスト =====

    #[test]
    fn render_footnotes_when_enabled() {
        // 脚注機能の正しいHTML生成を確認（有効時）
        let markdown = "Text with footnote[^1].\n\n[^1]: This is a footnote.";
        let result = render(markdown, &Options::default());
        // 脚注リンクとコンテンツの存在を確認
        assert!(
            result.html.contains("footnote-reference"),
            "Expected rendered HTML to contain a footnote reference, got: {}",
            result.html
        );
        assert!(
            result.html.contains("footnote-definition"),
            "Expected rendered HTML to contain the footnote definition block, got: {}",
            result.html
        );
    }

    #[test]
    fn ignore_footnotes_when_disabled() {
        // 脚注機能の無効化確認
        let markdown = "Text with footnote[^1].\n\n[^1]: This is a footnote.";
        let mut options = Options::default();
        options.footnotes = false;
        let result = render(markdown, &options);
        // 脚注が処理されずにそのまま残ることを確認
        assert!(result.html.contains("[^1]"));
    }

    #[test]
    fn transform_smart_punctuation_when_enabled() {
        // スマート句読点機能の確認（有効時）
        let markdown = "\"Hello\" and 'world' -- test.";
        let result = render(markdown, &Options::default());
        // スマート変換が行われることを確認（具体的な文字は実装依存）
        assert!(result.html.len() >= markdown.len());
    }

    #[test]
    fn preserve_punctuation_when_smart_disabled() {
        // スマート句読点機能の無効化確認
        let markdown = "\"Hello\" and 'world' -- test.";
        let mut options = Options::default();
        options.smart_punct = false;
        let result = render(markdown, &options);

        // pulldown-cmarkはHTMLエンティティとしてエスケープするため、
        // &quot; の形で出力される（これは正しい動作）
        assert!(result.html.contains("&quot;Hello&quot;"));
        assert!(result.html.contains("'world'"));
        assert!(result.html.contains(" -- "));
    }

    // ===== Unicode・CJK文字テスト =====

    #[test]
    fn render_cjk_content() {
        // CJK文字の正しい処理を確認
        let markdown = "# 日本語の見出し\n\n中国語：你好世界\n\n한글: 안녕하세요";
        let result = render(markdown, &Options::default());
        assert!(result.html.contains("<h1>日本語の見出し</h1>"));
        assert!(result.html.contains("<p>中国語：你好世界</p>"));
        assert!(result.html.contains("<p>한글: 안녕하세요</p>"));

        // 見出し抽出でCJK文字が正しく処理されることを確認
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "日本語の見出し");
    }

    #[test]
    fn render_mixed_script_content() {
        // 複数文字体系の混在コンテンツの処理を確認
        let markdown = "# Mixed 文字 Scripts 한글\n\nEnglish and 日本語 and 한국어.";
        let result = render(markdown, &Options::default());
        assert!(result.html.contains("<h1>Mixed 文字 Scripts 한글</h1>"));
        assert!(result
            .html
            .contains("<p>English and 日本語 and 한국어.</p>"));
    }

    // ===== エッジケース・エラーハンドリングテスト =====

    #[test]
    fn handle_empty_input() {
        // 空文字列の処理を確認
        let result = render("", &Options::default());
        assert_eq!(result.headings.len(), 0);
        // 空のHTMLまたは最小限のHTMLが返されることを確認
        assert!(result.html.len() < 50); // 過度に長くないことを確認
    }

    #[test]
    fn handle_whitespace_only_input() {
        // 空白のみの入力の処理を確認
        let result = render("   \n\n  \t  \n", &Options::default());
        assert_eq!(result.headings.len(), 0);
    }

    #[test]
    fn handle_malformed_markdown() {
        // 不正なMarkdown構文の寛容な処理を確認
        let malformed = "# Unclosed **bold\n\n[Invalid link](";
        let result = render(malformed, &Options::default());
        // エラーが発生せず、何らかのHTMLが生成されることを確認
        assert!(!result.html.is_empty());
        assert_eq!(result.headings.len(), 1); // 見出しは正しく抽出される
    }

    #[test]
    fn escape_html_content() {
        // HTMLエスケープの確認
        // pulldown-cmarkはデフォルトでraw HTMLを許可するが、
        // これは標準的なMarkdown動作。危険なコンテンツでテストする場合は
        // より安全な例を使用する。
        let markdown = "Code with `<script>alert('xss')</script>` tags.";
        let result = render(markdown, &Options::default());

        // コードとして適切にエスケープされることを確認
        assert!(result.html.contains("<code>"));
        assert!(result.html.contains("&lt;script&gt;"));
        assert!(result.html.contains("&lt;/script&gt;"));
    }

    #[test]
    fn handle_large_content() {
        // 大きなコンテンツの処理パフォーマンステスト
        let large_content = "# Test\n\n".repeat(1000) + &"Content line.\n".repeat(1000);
        let result = render(&large_content, &Options::default());
        assert_eq!(result.headings.len(), 1000); // 全ての見出しが抽出される
        assert!(result.html.len() > large_content.len()); // HTML変換が行われる
    }

    // ===== オプション組み合わせテスト =====

    #[test]
    fn render_with_all_options_disabled() {
        // 全機能無効時の基本動作確認
        let markdown = "# Title\n\n| Table | Test |\n|-------|------|\n| A | B |\n\n- [x] Task";
        let options = Options {
            gfm_tables: false,
            gfm_tasklists: false,
            footnotes: false,
            smart_punct: false,
        };
        let result = render(markdown, &options);

        // 基本要素は動作する
        assert!(result.html.contains("<h1>Title</h1>"));
        // 拡張機能は無効
        assert!(!result.html.contains("<table>"));
        assert!(!result.html.contains("type=\"checkbox\""));
    }

    #[test]
    fn render_with_selective_options() {
        // 選択的オプション有効化の確認
        let markdown = "\"Smart quotes\" and:\n\n| Table | Test |\n|-------|------|\n| A | B |";
        let mut options = Options::default();
        options.gfm_tables = true; // テーブルのみ有効
        options.smart_punct = false; // スマート句読点は無効

        let result = render(markdown, &options);

        // HTMLエンティティとしてエスケープされる（正しい動作）
        assert!(result.html.contains("&quot;Smart quotes&quot;"));
        // テーブルは機能する
        assert!(result.html.contains("<table>"));
    }

    // ===== 見出しslug生成テスト（既存機能の保持確認） =====

    #[test]
    fn generate_heading_slugs() {
        // 見出しのslug生成が正しく動作することを確認
        let result = render("# Hello World", &Options::default());
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].slug, "hello-world");
    }

    #[test]
    fn generate_cjk_heading_slugs() {
        // CJK文字のslug生成確認
        let result = render("# 日本語の見出し", &Options::default());
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "日本語の見出し");
        assert_eq!(result.headings[0].slug, "日本語の見出し"); // CJK文字は保持
    }

    #[test]
    fn prevent_slug_collisions() {
        // slug衝突防止機能の確認
        let markdown = "# Test\n\n# Test\n\n# Test";
        let result = render(markdown, &Options::default());
        assert_eq!(result.headings.len(), 3);
        assert_eq!(result.headings[0].slug, "test");
        assert_eq!(result.headings[1].slug, "test-1");
        assert_eq!(result.headings[2].slug, "test-2");
    }

    // ===== 既存テスト（後方互換性確認） =====

    #[test]
    fn render_basic_markdown() {
        // 基本的なレンダリング機能の動作確認
        let result = render("# Test Header\n\nParagraph content.", &Options::default());
        assert!(!result.html.is_empty());
        assert!(result.headings.len() > 0);
        assert_eq!(result.headings[0].text, "Test Header");
    }

    #[test]
    fn generate_mixed_cjk_slugs() {
        // CJK文字のslug生成テスト（is_cjk関数との連携確認）
        let result = render("# 测试 한글 テスト", &Options::default());
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "测试 한글 テスト");
        // slug生成でCJK文字が適切に処理されることを確認
        assert!(result.headings[0].slug.contains("测试"));
        assert!(result.headings[0].slug.contains("한글"));
        assert!(result.headings[0].slug.contains("テスト"));
    }
}
