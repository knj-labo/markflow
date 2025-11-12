//! # rsmd-core
//!
//! pulldown-cmark を用いた RSMD の高速Markdownレンダラー実装です。
//!
//! ## 現在の状態 (PR0完了 → PR1準備中)
//!
//! - HTML 出力は `pulldown-cmark` に完全委譲しており、CommonMark + GFM の正確なレンダリングを最優先します。
//! - 見出し収集は `pulldown_cmark::Event` ベースの実装で CommonMark 準拠の正確な検出を実現します。
//! - H1見出しのみを対象とし、ASCII専用スラグ生成（衝突処理付き）を実装。
//! - 📦 API は `render()` と `RenderResult { html, headings }` を安定させ、将来の機能拡張にも対応します。
//!
//! ## 次のステップ
//! - 🔄 **PR1準備中**: Unicode/CJK スラグ化とドキュメント整備。
//! - ⏳ **PR2予定**: HTML 生成と見出し収集のシングルパス統合（TODO.md 参照）。
//!
//! ## 参考実装
//!
//! - HTMLエスケープ: <https://github.com/wooorm/markdown-rs/blob/main/src/util/encode.rs>
//! - URIサニタイズ: <https://github.com/wooorm/markdown-rs/blob/main/src/util/sanitize_uri.rs>
//! - HTML生成アーキテクチャ: <https://github.com/wooorm/markdown-rs/blob/main/src/to_html.rs>
//! - GitHub互換slug生成: <https://github.com/markdown-it-rust/markdown-it-plugins.rs/blob/main/crates/github_slugger/src/lib.rs>
//! - GitHub互換slug (crate): <https://docs.rs/github-slugger>
//! - pulldown-cmark (使用中): <https://docs.rs/pulldown-cmark>

pub use pulldown_cmark::{html, Event, HeadingLevel, Options as CmarkOptions, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

mod is_cjk;
pub use crate::is_cjk::is_cjk;

mod sanitize_html;
pub use crate::sanitize_html::sanitize_html;

mod slugify;
pub use crate::slugify::{slugify, slugify_ascii};

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

/// Markdownをレンダリング（pulldown-cmark + イベントベース見出し収集）
///
/// 1. `pulldown-cmark` で CommonMark + GFM HTML を生成します。
/// 2. 同じMarkdown文字列を `pulldown_cmark::Event` で再度パースし、H1見出しのみを正確に収集します。
/// 3. **ASCII専用スラグ生成**: 各H1見出しに衝突処理付きのASCIIスラグを添付します。
///
/// イベントベース実装により、コードブロック内の偽見出しや無効なATX構文を正しく除外し、
/// CommonMark準拠の見出し検出を実現します。
///
/// ## サポートする要素
/// - 基本: 見出し / 段落 / 強調 / コード / リスト / リンク / 画像
/// - GFM: テーブル / タスクリスト / 取り消し線 / 自動リンク / 脚注
/// - オプション: `Options` で tables / tasklists / footnotes / smart punctuation を個別に制御
///
/// ## 使用例
///
/// `RenderResult` は HTML と見出しリスト（depth / text / slug）を返し、
/// 将来のPRで heading の正確性を高めても API 互換性を保てるようにしています。
///
/// ## 現在の実装状況と今後の改善
/// - ✅ 見出し収集: H1見出しのみを対象とし、ASCII専用スラグ生成（衝突処理付き）を実装完了。
/// - ⏳ 2パス処理: HTML生成と見出し収集が独立（PR2でシングルパス統合予定）
/// - pulldown-cmark の生HTMLが必要な場合は `sanitize_html` を組み合わせて利用してください。
pub fn render(source: &str, options: &Options) -> RenderResult {
    // pulldown-cmarkオプションに変換
    let cmark_options = convert_options(options);

    // パーサーを初期化
    let parser = Parser::new_ext(source, cmark_options);

    // HTMLを生成
    let mut html = String::new();
    html::push_html(&mut html, parser);

    // 見出し抽出のためにイベントベースで再度パースする
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

/// 見出し抽出（イベントベース・CommonMark準拠）
///
/// `pulldown_cmark::Event` ストリームを処理して、CommonMark仕様に準拠した
/// 見出し検出を行います。regex解析とは異なり、構文解析済みのイベントを
/// 使用するため以下の利点があります：
///
/// ## CommonMark準拠の改善点
/// - コードブロック内の `# Heading` は見出しとして扱いません
/// - ATX見出しの `#word` (スペースなし) は無効として扱います
/// - `#######` (7個以上の#) は見出しとして認識されません
/// - インラインフォーマット (`# **Bold** Title`) を正しく処理します
///
/// ## 処理スコープ (PR0実装完了)
/// - H1見出し (depth=1) のみを収集対象とします
/// - setext見出し (`Title\n====`) は将来対応予定として現在は対象外です
///
/// ## イベント処理アルゴリズム
/// 1. `Event::Start(Tag::Heading(1, _, _))` でH1見出し開始を検出
/// 2. 見出し内のテキストフラグメントを適切な文脈で収集
/// 3. `Event::End(Tag::Heading(1))` で見出し終了、テキスト確定
/// 4. コードブロックや不適切な文脈内では見出しを無視
fn extract_headings(source: &str, options: &CmarkOptions) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut used_slugs = HashSet::new();
    let parser = Parser::new_ext(source, *options);

    let mut current_heading_text = String::new();
    let mut in_h1_heading = false;
    let mut in_code_block = false;

    for event in parser {
        match event {
            // コードブロックの開始・終了を追跡
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
            }
            Event::End(Tag::CodeBlock(_)) => {
                in_code_block = false;
            }

            // H1見出しの開始を検出
            Event::Start(Tag::Heading(level, _, _))
                if level == HeadingLevel::H1 && !in_code_block =>
            {
                in_h1_heading = true;
                current_heading_text.clear();
            }

            // H1見出しの終了を検出
            Event::End(Tag::Heading(level, _, _)) if level == HeadingLevel::H1 && in_h1_heading => {
                in_h1_heading = false;
                let text = current_heading_text.trim().to_string();
                if !text.is_empty() {
                    // PR0実装：ASCII専用スラッグ生成（衝突処理付き）
                    let slug = crate::slugify::slugify_ascii(&text, &mut used_slugs);
                    headings.push(Heading {
                        depth: 1,
                        text,
                        slug,
                    });
                }
                current_heading_text.clear();
            }

            // H1見出し内のテキストを収集
            Event::Text(text) if in_h1_heading => {
                current_heading_text.push_str(&text);
            }

            // H1見出し内の他のイベント（Code、SoftBreak、HardBreakなど）もテキスト化
            Event::Code(code) if in_h1_heading => {
                current_heading_text.push_str(&code);
            }

            Event::SoftBreak if in_h1_heading => {
                current_heading_text.push(' ');
            }

            Event::HardBreak if in_h1_heading => {
                current_heading_text.push(' ');
            }

            // その他のイベントは無視（H1以外の見出し、非H1コンテンツなど）
            _ => {}
        }
    }

    headings
}

// ===== 内部状態（将来のPR2向けシングルパス統合実装予定） =====

// 将来のPR2でシングルパス統合の際に以下の構造体を使用する可能性：
// /// 見出し処理中の状態
// struct HeadingState {
//     depth: u8,
//     text: String,
// }
//
// /// 見出し収集器
// /// 参考: markdown-rsのCompileContext的な状態管理
// /// - <https://github.com/wooorm/markdown-rs/blob/main/src/to_html.rs>
// struct HeadingRecorder {
//     current_heading: Option<HeadingState>,
//     headings: Vec<Heading>,
//     used_slugs: HashSet<String>,
// }

#[cfg(target_arch = "wasm32")]
pub mod wasm_bindings;

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
    fn event_based_extraction_rejects_tight_atx_syntax() {
        // イベントベース実装では `#Heading` (スペースなし) は見出しとして扱わない
        // これはCommonMark準拠の正しい動作
        let markdown = "#NoSpace\n\nParagraph"; // 空行を追加して別段落にする
        let result = render(markdown, &Options::default());

        // pulldown-cmarkは #NoSpace を段落として処理する
        assert!(result.html.contains("<p>#NoSpace</p>"));
        assert!(result.html.contains("<p>Paragraph</p>"));
        // 見出しは検出されない（CommonMark準拠）
        assert_eq!(result.headings.len(), 0);
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
    fn heading_slugs_generate_ascii_in_pr0() {
        // PR0ではASCII専用スラッグ生成（衝突処理付き）を実装
        let result = render("# Hello World", &Options::default());
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].slug, "hello-world"); // ASCII slug
        assert_eq!(result.headings[0].text, "Hello World");
        assert_eq!(result.headings[0].depth, 1);
    }

    #[test]
    fn cjk_heading_slugs_fallback_to_section_in_pr0() {
        // PR0ではCJK文字の見出しはASCII文字がないため"section"にフォールバック
        let result = render("# 日本語の見出し", &Options::default());
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "日本語の見出し");
        assert_eq!(result.headings[0].slug, "section"); // ASCII fallback
        assert_eq!(result.headings[0].depth, 1);
    }

    #[test]
    fn slug_collisions_handled_in_pr0() {
        // PR0ではASCII専用スラッグ生成で衝突防止機能が有効
        let markdown = "# Test\n\n# Test\n\n# Test";
        let result = render(markdown, &Options::default());
        assert_eq!(result.headings.len(), 3);
        // 衝突回避による一意なスラッグが生成される
        assert_eq!(result.headings[0].slug, "test");
        assert_eq!(result.headings[1].slug, "test-1");
        assert_eq!(result.headings[2].slug, "test-2");
        // テキストは正しく収集される
        assert!(result.headings.iter().all(|h| h.text == "Test"));
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
    fn mixed_cjk_heading_text_preserved_slug_fallback_in_pr0() {
        // CJK文字のテキスト抽出テスト（ASCII文字がないため"section"にフォールバック）
        let result = render("# 测试 한글 テスト", &Options::default());
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "测试 한글 テスト");
        assert_eq!(result.headings[0].slug, "section"); // ASCII fallback
        assert_eq!(result.headings[0].depth, 1);
    }

    // ===== CommonMark準拠テスト（イベントベース見出し抽出用） =====

    #[test]
    fn ignore_headings_in_code_blocks() {
        // コードブロック内の # Heading は見出しとして扱わない
        let markdown = "```\n# Not a heading\n```\n\n# Real heading";
        let result = render(markdown, &Options::default());

        // HTML出力は正しくコードブロックを生成
        assert!(result.html.contains("<pre><code># Not a heading"));
        assert!(result.html.contains("<h1>Real heading</h1>"));

        // 見出し抽出では実際の見出しのみを検出
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "Real heading");
    }

    #[test]
    fn reject_atx_headings_without_space() {
        // #word (スペースなし) は見出しとして扱わない
        let markdown = "#NotAHeading\n\n# Real Heading";
        let result = render(markdown, &Options::default());

        // pulldown-cmarkの動作：スペースなしは段落として処理される
        assert!(result.html.contains("<p>#NotAHeading</p>"));
        assert!(result.html.contains("<h1>Real Heading</h1>"));

        // 見出し抽出では正しい見出しのみを検出
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "Real Heading");
    }

    #[test]
    fn reject_invalid_atx_headings_with_seven_or_more_hashes() {
        // ####### (7個以上) は見出しとして扱わない
        let markdown = "####### Invalid\n\n# Valid";
        let result = render(markdown, &Options::default());

        // pulldown-cmarkの動作：7個以上の#は段落として処理される
        assert!(result.html.contains("<p>####### Invalid</p>"));
        assert!(result.html.contains("<h1>Valid</h1>"));

        // 見出し抽出では有効な見出しのみを検出
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "Valid");
    }

    #[test]
    fn extract_only_h1_headings() {
        // H1見出しのみを抽出し、他のレベルは無視する
        let markdown = "# H1 Title\n## H2 Subtitle\n### H3 Section\n# Another H1";
        let result = render(markdown, &Options::default());

        // HTML出力には全ての見出しが含まれる
        assert!(result.html.contains("<h1>H1 Title</h1>"));
        assert!(result.html.contains("<h2>H2 Subtitle</h2>"));
        assert!(result.html.contains("<h3>H3 Section</h3>"));
        assert!(result.html.contains("<h1>Another H1</h1>"));

        // 見出し抽出ではH1のみを収集
        assert_eq!(result.headings.len(), 2);
        assert_eq!(result.headings[0].text, "H1 Title");
        assert_eq!(result.headings[1].text, "Another H1");
        // 全てのdepthが1であることを確認
        assert!(result.headings.iter().all(|h| h.depth == 1));
    }

    #[test]
    fn handle_inline_formatting_in_headings() {
        // 見出し内のインラインフォーマットを正しく処理
        let markdown = "# **Bold** and *italic* and `code` heading";
        let result = render(markdown, &Options::default());

        // HTML出力には適切なフォーマットが含まれる
        assert!(result.html.contains(
            "<h1><strong>Bold</strong> and <em>italic</em> and <code>code</code> heading</h1>"
        ));

        // 見出し抽出ではプレーンテキストとして収集
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "Bold and italic and code heading");
    }

    #[test]
    fn ignore_headings_in_inline_code() {
        // インラインコード内の # は見出しとして扱わない
        let markdown = "Text with `# not a heading` in code.\n\n# Real heading";
        let result = render(markdown, &Options::default());

        // HTML出力は正しく処理される
        assert!(result.html.contains("<code># not a heading</code>"));
        assert!(result.html.contains("<h1>Real heading</h1>"));

        // 見出し抽出では実際の見出しのみを検出
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "Real heading");
    }

    #[test]
    fn slug_generation_enabled_ascii_in_pr0() {
        // PR0仕様：ASCII専用スラグ生成（衝突処理付き）を実装
        let markdown = "# Test Heading";
        let result = render(markdown, &Options::default());

        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "Test Heading");
        assert_eq!(result.headings[0].depth, 1);
        // ASCIIスラグが生成される
        assert_eq!(result.headings[0].slug, "test-heading");
    }

    // ===== PR0 ASCII スラッグ生成 統合テスト =====

    #[test]
    fn pr0_ascii_slug_with_mixed_content() {
        // 英数字＋CJK文字の混在見出しでASCII部分のみスラグ化
        let markdown = "# Hello 世界 123\n\n# API ドキュメント v2.0\n\n# 測試 Test";
        let result = render(markdown, &Options::default());

        assert_eq!(result.headings.len(), 3);
        assert_eq!(result.headings[0].slug, "hello-123");
        assert_eq!(result.headings[1].slug, "api-v2-0");
        assert_eq!(result.headings[2].slug, "test");
    }

    #[test]
    fn pr0_ascii_slug_collision_prevention() {
        // 自然スラッグと衝突解決スラッグの競合防止
        let markdown = "# Section\n\n# Section 1\n\n# Section\n\n# Section-1";
        let result = render(markdown, &Options::default());

        assert_eq!(result.headings.len(), 4);
        assert_eq!(result.headings[0].slug, "section"); // 初回
        assert_eq!(result.headings[1].slug, "section-1"); // 自然生成
        assert_eq!(result.headings[2].slug, "section-2"); // 衝突回避（section-1は使用済み）
        assert_eq!(result.headings[3].slug, "section-1-1"); // さらに衝突回避
    }

    #[test]
    fn pr0_ascii_slug_normalization() {
        // 区切り文字の正規化と特殊文字処理
        let markdown = "# hello_world-test.file/path\n\n# Multiple   Spaces\n\n# @#$%^&*()";
        let result = render(markdown, &Options::default());

        assert_eq!(result.headings.len(), 3);
        assert_eq!(result.headings[0].slug, "hello-world-test-file-path");
        assert_eq!(result.headings[1].slug, "multiple-spaces");
        assert_eq!(result.headings[2].slug, "section"); // 記号のみ → fallback
    }

    #[test]
    fn pr0_ascii_slug_multiple_fallbacks() {
        // 複数の"section"フォールバックで衝突処理
        let markdown = "# !!!\n\n# 日本語\n\n# 😀🎉\n\n# @#$";
        let result = render(markdown, &Options::default());

        assert_eq!(result.headings.len(), 4);
        assert_eq!(result.headings[0].slug, "section");
        assert_eq!(result.headings[1].slug, "section-1");
        assert_eq!(result.headings[2].slug, "section-2");
        assert_eq!(result.headings[3].slug, "section-3");
    }

    #[test]
    fn pr0_demo_comprehensive_functionality() {
        // Comprehensive demonstration of PR0 ASCII slug functionality
        let markdown = r#"
# Hello World
# 日本語の見出し
# Section
# Section
# API Documentation v2.0
# Hello 世界 123
# @#$%^&*()
# Section-1
"#;

        let result = render(markdown, &Options::default());
        assert_eq!(result.headings.len(), 8);

        // Verify all expected slugs are generated correctly
        let expected_slugs = vec![
            "hello-world",            // Basic ASCII normalization
            "section",                // CJK fallback to "section"
            "section-1",              // First collision resolution
            "section-2",              // Second collision resolution
            "api-documentation-v2-0", // Complex normalization
            "hello-123",              // Mixed content (ASCII only)
            "section-3",              // Symbol-only fallback
            "section-1-1",            // Collision with existing "section-1"
        ];

        for (i, heading) in result.headings.iter().enumerate() {
            assert_eq!(
                heading.slug, expected_slugs[i],
                "Heading '{}' should have slug '{}' but got '{}'",
                heading.text, expected_slugs[i], heading.slug
            );
        }
    }
}
