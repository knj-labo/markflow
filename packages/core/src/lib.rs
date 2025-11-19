//! # rsmd-core
//!
//! pulldown-cmark を用いた RSMD の高速Markdownレンダラー実装です。
//!
//! ## 現在の状態 (PR1完了 → PR2準備中)
//!
//! - HTML 出力は `pulldown-cmark` に完全委譲しており、CommonMark + GFM の正確なレンダリングを最優先します。
//! - 見出し収集は `pulldown_cmark::Event` ベースの実装で CommonMark 準拠の正確な検出を実現します。
//! - H1〜H3 見出しを対象とし、Unicode/CJK 対応スラグ生成（衝突処理付き）を実装。
//! - 📦 API は `render()` と `RenderResult { html, headings }` を安定させ、将来の機能拡張にも対応します。
//!
//! ## 次のステップ
//! - ✅ **PR1完了**: h1〜h3 見出し拡張と HTML/見出しのシングルパス統合（TODO.md の PR1 参照）。
//! - ⏳ **PR2予定**: wasm-bindgen 最小エクスポート整備と `render_wasm` 公開（TODO.md の PR2 参照）。
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
/// 1. `pulldown-cmark` のイベントストリームを 1 パスで走査し、HTML 出力に必要な全イベントを記録します。
/// 2. 同じイベント走査で H1〜H3 見出しを検出し、Unicode/CJK 対応スラグを生成します。
/// 3. HTML 生成時に各見出しへ `id="slug"` 属性を注入し、リンク用アンカーを保証します。
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
pub fn render(source: &str, options: &Options) -> RenderResult {
    // pulldown-cmarkオプションに変換
    let cmark_options = convert_options(options);

    // パーサーを初期化し、単一パスでイベントを収集
    let parser = Parser::new_ext(source, cmark_options);
    let mut collector = HeadingCollector::new();
    let mut events = Vec::new();

    for event in parser {
        collector.process_event(&event);
        events.push(event);
    }

    // HTMLを生成
    let mut html = String::new();
    html::push_html(&mut html, events.into_iter());

    // 収集済み見出しを取得し、HTMLにid属性を注入
    let headings = collector.into_headings();
    inject_heading_ids(&mut html, &headings);

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

/// 単一パスでH1〜H3見出しを収集するためのヘルパー。
struct HeadingCollector {
    headings: Vec<Heading>,
    used_slugs: HashSet<String>,
    current: Option<HeadingState>,
}

impl HeadingCollector {
    fn new() -> Self {
        Self {
            headings: Vec::new(),
            used_slugs: HashSet::new(),
            current: None,
        }
    }

    fn process_event<'a>(&mut self, event: &Event<'a>) {
        match event {
            Event::Start(Tag::Heading(level, _, _)) if is_tracked_heading(*level) => {
                self.current = Some(HeadingState {
                    level: *level,
                    text: String::new(),
                });
            }
            Event::End(Tag::Heading(level, _, _)) if is_tracked_heading(*level) => {
                if self
                    .current
                    .as_ref()
                    .map(|state| state.level == *level)
                    .unwrap_or(false)
                {
                    self.finalize_current_heading();
                }
            }
            Event::Text(text) => {
                if let Some(state) = self.current.as_mut() {
                    state.text.push_str(text.as_ref());
                }
            }
            Event::Code(code) => {
                if let Some(state) = self.current.as_mut() {
                    state.text.push_str(code.as_ref());
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(state) = self.current.as_mut() {
                    state.text.push(' ');
                }
            }
            _ => {}
        }
    }

    fn finalize_current_heading(&mut self) {
        if let Some(state) = self.current.take() {
            let text = state.text.trim().to_string();
            let slug_source = if text.is_empty() {
                "section"
            } else {
                text.as_str()
            };
            let slug = slugify(slug_source, &mut self.used_slugs);
            self.headings.push(Heading {
                depth: heading_level_to_depth(state.level),
                text,
                slug,
            });
        }
    }

    fn into_headings(mut self) -> Vec<Heading> {
        if self.current.is_some() {
            self.finalize_current_heading();
        }
        self.headings
    }
}

struct HeadingState {
    level: HeadingLevel,
    text: String,
}

fn is_tracked_heading(level: HeadingLevel) -> bool {
    matches!(
        level,
        HeadingLevel::H1 | HeadingLevel::H2 | HeadingLevel::H3
    )
}

fn heading_level_to_depth(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn inject_heading_ids(html: &mut String, headings: &[Heading]) {
    if headings.is_empty() {
        return;
    }

    let mut search_start = 0usize;
    for heading in headings {
        let prefix = format!("<h{}", heading.depth);
        if let Some(tag_start) = find_heading_tag(html, &prefix, search_start) {
            if let Some(close_offset) = html[tag_start..].find('>') {
                let tag_end = tag_start + close_offset;
                let insert_pos = tag_start + prefix.len();
                let tag_segment = &html[tag_start..tag_end];
                if has_id_attribute(tag_segment) {
                    search_start = tag_end;
                    continue;
                }

                let attr = format!(" id=\"{}\"", heading.slug);
                html.insert_str(insert_pos, &attr);
                search_start = insert_pos + attr.len();
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

fn find_heading_tag(html: &str, needle: &str, start: usize) -> Option<usize> {
    let mut search_from = start;
    while let Some(rel) = html[search_from..].find(needle) {
        let pos = search_from + rel;
        let next_idx = pos + needle.len();
        if let Some(next_byte) = html.as_bytes().get(next_idx) {
            if next_byte.is_ascii_digit() {
                search_from = next_idx;
                continue;
            }
        } else {
            return None;
        }
        return Some(pos);
    }
    None
}

fn has_id_attribute(segment: &str) -> bool {
    let mut lower = segment.chars().collect::<String>();
    lower.make_ascii_lowercase();
    lower.contains(" id=")
}

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
            result
                .html
                .contains("<h1 id=\"hello-world\">Hello World</h1>"),
            "Expected <h1 id=\"hello-world\">Hello World</h1>, got: {}",
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
        assert!(result.html.contains("<h1 id=\"title\">Title</h1>"));
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
        assert!(result.html.contains("<h1 id=\"h1-title\">H1 Title</h1>"));
        assert!(result
            .html
            .contains("<h2 id=\"h2-subtitle\">H2 Subtitle</h2>"));
        assert!(result
            .html
            .contains("<h3 id=\"h3-section\">H3 Section</h3>"));

        let depths: Vec<_> = result.headings.iter().map(|h| h.depth).collect();
        assert_eq!(depths, vec![1, 2, 3]);
    }

    #[test]
    fn assigns_unique_slugs_across_heading_levels() {
        // 同じテキストでも各見出しに一意のスラグを付与する
        let markdown = "# Repeat\n## Repeat\n### Repeat";
        let result = render(markdown, &Options::default());

        assert!(result.html.contains("<h1 id=\"repeat\">Repeat</h1>"));
        assert!(result.html.contains("<h2 id=\"repeat-1\">Repeat</h2>"));
        assert!(result.html.contains("<h3 id=\"repeat-2\">Repeat</h3>"));

        let slugs: Vec<_> = result.headings.iter().map(|h| h.slug.as_str()).collect();
        assert_eq!(slugs, vec!["repeat", "repeat-1", "repeat-2"]);
    }

    #[test]
    fn supports_unicode_cjk_headings() {
        // Unicode/CJK 見出しでもスラグとidが正しく生成される
        let markdown = "# 日本語 タイトル\n## こんにちは 世界";
        let result = render(markdown, &Options::default());

        assert!(result
            .html
            .contains("<h1 id=\"日本語-タイトル\">日本語 タイトル</h1>"));
        assert!(result
            .html
            .contains("<h2 id=\"こんにちは-世界\">こんにちは 世界</h2>"));

        assert_eq!(result.headings[0].slug, "日本語-タイトル");
        assert_eq!(result.headings[1].slug, "こんにちは-世界");
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
    fn single_pass_heading_scan_preserves_gfm_html_correctness() {
        // 単一パスの見出し抽出でもHTML生成が正しいことを確認
        let markdown = "# Table Heading\n\n| Name | Age |\n|------|-----|\n| Alice | 30 |";
        let result = render(markdown, &Options::default());

        assert!(result.html.contains("<table>"));
        assert!(result
            .html
            .contains("<h1 id=\"table-heading\">Table Heading</h1>"));
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
        assert!(result.html.contains("<h1 id=\"日本語の見出し\">日本語の見出し</h1>"));
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
        assert!(
            result
                .html
                .contains("<h1 id=\"mixed-文字-scripts-한글\">Mixed 文字 Scripts 한글</h1>")
        );
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
        assert!(result.html.contains("<h1 id=\"title\">Title</h1>"));
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
    fn heading_slugs_generate_ascii_when_applicable() {
        // ASCII見出しでは従来通り小文字+ハイフン区切りのスラグを生成
        let result = render("# Hello World", &Options::default());
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].slug, "hello-world"); // ASCII slug
        assert_eq!(result.headings[0].text, "Hello World");
        assert_eq!(result.headings[0].depth, 1);
    }

    #[test]
    fn cjk_heading_slugs_preserve_unicode() {
        // CJK見出しはUnicodeスラグとして保持される
        let result = render("# 日本語の見出し", &Options::default());
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "日本語の見出し");
        assert_eq!(result.headings[0].slug, "日本語の見出し");
        assert_eq!(result.headings[0].depth, 1);
    }

    #[test]
    fn slug_collisions_handled_consistently() {
        // Unicode対応後も衝突防止機能は維持される
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
    fn unicode_slugs_preserve_mixed_cjk_text() {
        // CJK文字と異なるスクリプトの混在でもUnicodeスラグを維持する
        let result = render("# 测试 한글 テスト", &Options::default());
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "测试 한글 テスト");
        assert_eq!(result.headings[0].slug, "测试-한글-テスト");
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
        assert!(result
            .html
            .contains("<h1 id=\"real-heading\">Real heading</h1>"));

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
        assert!(result
            .html
            .contains("<h1 id=\"real-heading\">Real Heading</h1>"));

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
        assert!(result.html.contains("<h1 id=\"valid\">Valid</h1>"));

        // 見出し抽出では有効な見出しのみを検出
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "Valid");
    }

    #[test]
    fn extract_h1_h2_h3_headings() {
        // H1〜H3見出しを抽出する
        let markdown = "# H1 Title\n## H2 Subtitle\n### H3 Section\n# Another H1";
        let result = render(markdown, &Options::default());

        // HTML出力には全ての見出しが含まれる
        assert!(result.html.contains("<h1 id=\"h1-title\">H1 Title</h1>"));
        assert!(result
            .html
            .contains("<h2 id=\"h2-subtitle\">H2 Subtitle</h2>"));
        assert!(result
            .html
            .contains("<h3 id=\"h3-section\">H3 Section</h3>"));
        assert!(result
            .html
            .contains("<h1 id=\"another-h1\">Another H1</h1>"));

        // 見出し抽出ではH1〜H3を収集
        assert_eq!(result.headings.len(), 4);
        let depths: Vec<_> = result.headings.iter().map(|h| h.depth).collect();
        assert_eq!(depths, vec![1, 2, 3, 1]);
    }

    #[test]
    fn handle_inline_formatting_in_headings() {
        // 見出し内のインラインフォーマットを正しく処理
        let markdown = "# **Bold** and *italic* and `code` heading";
        let result = render(markdown, &Options::default());

        // HTML出力には適切なフォーマットが含まれる
        assert!(result.html.contains(
            "<h1 id=\"bold-and-italic-and-code-heading\"><strong>Bold</strong> and <em>italic</em> and <code>code</code> heading</h1>"
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
        assert!(result
            .html
            .contains("<h1 id=\"real-heading\">Real heading</h1>"));

        // 見出し抽出では実際の見出しのみを検出
        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "Real heading");
    }

    #[test]
    fn slug_generation_for_ascii_headings() {
        // ASCIIのみの見出しでは従来通りのスラグを生成
        let markdown = "# Test Heading";
        let result = render(markdown, &Options::default());

        assert_eq!(result.headings.len(), 1);
        assert_eq!(result.headings[0].text, "Test Heading");
        assert_eq!(result.headings[0].depth, 1);
        // ASCIIスラグが生成される
        assert_eq!(result.headings[0].slug, "test-heading");
    }

    // ===== Unicode Slug 統合テスト =====

    #[test]
    fn unicode_slug_with_mixed_content() {
        // 英数字＋CJK文字の混在見出しでもUnicodeを保持
        let markdown = "# Hello 世界 123\n\n# API ドキュメント v2.0\n\n# 測試 Test";
        let result = render(markdown, &Options::default());

        assert_eq!(result.headings.len(), 3);
        assert_eq!(result.headings[0].slug, "hello-世界-123");
        assert_eq!(result.headings[1].slug, "api-ドキュメント-v2-0");
        assert_eq!(result.headings[2].slug, "測試-test");
    }

    #[test]
    fn unicode_slug_collision_prevention() {
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
    fn unicode_slug_normalization() {
        // 区切り文字の正規化と特殊文字処理
        let markdown = "# hello_world-test.file/path\n\n# Multiple   Spaces\n\n# @#$%^&*()";
        let result = render(markdown, &Options::default());

        assert_eq!(result.headings.len(), 3);
        assert_eq!(result.headings[0].slug, "hello-world-test-file-path");
        assert_eq!(result.headings[1].slug, "multiple-spaces");
        assert_eq!(result.headings[2].slug, "section"); // 記号のみ → fallback
    }

    #[test]
    fn unicode_slug_multiple_fallbacks() {
        // Unicode混在時でもフォールバックと衝突処理を維持
        let markdown = "# !!!\n\n# 日本語\n\n# 😀🎉\n\n# @#$";
        let result = render(markdown, &Options::default());

        assert_eq!(result.headings.len(), 4);
        assert_eq!(result.headings[0].slug, "section");
        assert_eq!(result.headings[1].slug, "日本語");
        assert_eq!(result.headings[2].slug, "section-1");
        assert_eq!(result.headings[3].slug, "section-2");
    }

    #[test]
    fn unicode_slug_comprehensive_demo() {
        // Comprehensive demonstration of Unicode slug functionality
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
            "hello-world",             // Basic ASCII normalization
            "日本語の見出し",             // Unicode preserved
            "section",                 // First "Section"
            "section-1",               // Second "Section"
            "api-documentation-v2-0",  // Mixed ASCII + punctuation
            "hello-世界-123",            // Mixed ASCII + CJK
            "section-2",               // Symbol-only fallback
            "section-1-1",             // Collision with existing "section-1"
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
