use std::collections::HashSet;

/// Unicode保持のslug生成（Markdownの見出し用）
///
/// プレーンテキストからURL対応のslugを生成し、Unicode文字（特にCJK文字）を
/// 適切に処理します。以前に生成されたslugを追跡して一意性を保証し、
/// Markdownドキュメント用の衝突のないID生成を提供します。
///
/// ## 文字処理ルール
///
/// ### CJK文字（中国語・日本語・韓国語）
/// - 検出: `crate::is_cjk::is_cjk`を使用してCJK文字ブロックを識別
/// - 保持: CJK文字は大文字小文字変換せずそのまま保持
/// - 空白処理: CJK文字間の空白を除去して連続した表意文字列を作成
///   - 例: `"日本語 の 見出し"` → `"日本語の見出し"`
///
/// ### ASCII英数字
/// - 大文字小文字変換: すべてのASCII文字を小文字に変換
/// - 数字保持: 数字はそのまま維持
/// - 例: `"Hello World!"` → `"hello-world"`
///
/// ### Unicode英数字（ASCII以外）
/// - 大文字小文字変換: `ch.to_lowercase()`を使用した適切なUnicodeケースフォールディング
/// - 文字保持: アクセント記号付き文字や非ラテン文字を保持
/// - 例:
///   - `"Café"` → `"café"`
///   - `"Москва"` → `"москва"`
///
/// ### 区切り文字と空白
/// - 正規化: 空白、ハイフン、アンダースコア、ピリオド、スラッシュを
///   すべて単一のハイフンに正規化
/// - 重複除去: 連続する区切り文字を1つに集約
/// - トリミング: 先頭と末尾のハイフンを除去
/// - 例: `"hello_world-test.file/path"` → `"hello-world-test-file-path"`
///
/// ### その他の文字
/// - その他すべての文字（記号、句読点）は削除
///
/// ## フォールバックと一意性
///
/// - 空のフォールバック: 結果のslugが空の場合、`"section"`をデフォルトとして使用
/// - 一意性保証: 提供された`HashSet<String>`を使用して使用済みslugを追跡
///   - 初回出現: slugをそのまま使用
///   - 以降の出現: 一意になるまで`-1`、`-2`、`-3`等を追加
/// - 衝突防止: 自然に生成されるslug（例: "test-1"）と
///   衝突解決されたslug間の競合を防止
///
/// ## 実装注意事項
///
/// この実装はGitHubのslug生成にインスパイアされていますが、
/// より良いUnicodeサポート、特にCJK言語用に強化されています。
/// URL互換性と国際化要件のバランスを取っています。
///
/// ### 参考資料
/// - GitHub Slugger Rust実装: <https://github.com/markdown-it-rust/markdown-it-plugins.rs/blob/main/crates/github_slugger/src/lib.rs>
/// - Unicodeケースフォールディングと文字カテゴリー処理
pub fn slugify(text: &str, used_slugs: &mut HashSet<String>) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut last_hyphen = false;
    let mut prev_was_cjk = false;

    for ch in text.chars() {
        if crate::is_cjk::is_cjk(ch) {
            // 直前にCJK→空白で生成されたハイフンがある場合は除去する。
            if prev_was_cjk && last_hyphen {
                slug.pop();
            }
            slug.push(ch);
            last_hyphen = false;
            prev_was_cjk = true;
            continue;
        }

        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_hyphen = false;
            prev_was_cjk = false;
            continue;
        }

        // Support other Unicode alphanumeric characters beyond ASCII
        if ch.is_alphanumeric() {
            // Convert to lowercase if possible, otherwise use as-is
            for lower_ch in ch.to_lowercase() {
                slug.push(lower_ch);
            }
            last_hyphen = false;
            prev_was_cjk = false;
            continue;
        }

        if ch.is_whitespace() || matches!(ch, '-' | '_' | '.' | '/') {
            if !slug.is_empty() && !last_hyphen {
                slug.push('-');
                last_hyphen = true;
            }
            continue;
        }
        // その他の記号は無視。
    }

    if slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        slug.push_str("section");
    }

    // Ensure truly unique slugs by checking all possible variants
    let base = slug.clone();
    let mut candidate = slug;
    let mut n = 1;

    while used_slugs.contains(&candidate) {
        candidate = format!("{}-{}", base, n);
        n += 1;
    }

    used_slugs.insert(candidate.clone());
    candidate
}

/// ASCII専用スラッグ生成（Markdownの見出し用）
///
/// プレーンテキストからASCII文字のみを使用したURL対応のslugを生成します。
/// Unicode文字（CJK文字含む）は除去され、英数字と空白/句読点のみが処理されます。
/// 衝突処理機能により一意性を保証し、Markdownドキュメント用の安全なID生成を提供します。
///
/// ## 文字処理ルール
///
/// ### ASCII英数字
/// - 大文字小文字変換: すべてのASCII文字を小文字に変換
/// - 数字保持: ASCII数字（0-9）はそのまま維持
/// - 例: `"Hello World 123"` → `"hello-world-123"`
///
/// ### Unicode/CJK文字の除去
/// - 除去対象: ASCII範囲外のすべての文字（CJK文字、アクセント記号付き文字等）
/// - 例: `"Hello 世界"` → `"hello"`、`"Café"` → `"caf"`
///
/// ### 区切り文字と空白
/// - 正規化: 空白、ハイフン、アンダースコア、ピリオド、スラッシュを
///   すべて単一のハイフンに正規化
/// - 重複除去: 連続する区切り文字を1つに集約
/// - トリミング: 先頭と末尾のハイフンを除去
/// - 例: `"hello_world-test.file/path"` → `"hello-world-test-file-path"`
///
/// ### その他の文字
/// - ASCII範囲外の文字および記号・句読点は除去
///
/// ## フォールバックと一意性
///
/// - 空のフォールバック: 結果のスラッグが空の場合、`"section"`をデフォルトとして使用
/// - 一意性保証: 提供された`HashSet<String>`を使用して使用済みslugを追跡
///   - 初回出現: slugをそのまま使用
///   - 以降の出現: 一意になるまで`-1`、`-2`、`-3`等を追加
/// - 衝突防止: 自然に生成されるslug（例: "test-1"）と
///   衝突解決されたslug間の競合を防止
///
/// ## 実装注意事項
///
/// この実装はGitHubのslug生成にインスパイアされており、ASCII専用の
/// 安全なURL生成を優先しています。国際化よりもURL互換性を重視した設計です。
///
/// ### 参考資料
/// - GitHub Slugger Rust実装: <https://github.com/markdown-it-rust/markdown-it-plugins.rs/blob/main/crates/github_slugger/src/lib.rs>
/// - ASCIIフォールバック処理パターン
pub fn slugify_ascii(text: &str, used_slugs: &mut HashSet<String>) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut last_hyphen = false;

    for ch in text.chars() {
        // ASCII英数字のみを処理
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_hyphen = false;
            continue;
        }

        // ASCII区切り文字（空白、ハイフン、アンダースコア、ピリオド、スラッシュ）をハイフンに変換
        if ch.is_ascii_whitespace() || matches!(ch, '-' | '_' | '.' | '/') {
            if !slug.is_empty() && !last_hyphen {
                slug.push('-');
                last_hyphen = true;
            }
            continue;
        }
        // ASCII範囲外の文字（Unicode、記号等）は除去
    }

    // 末尾のハイフンを除去
    if slug.ends_with('-') {
        slug.pop();
    }

    // 空の場合はデフォルトを使用
    if slug.is_empty() {
        slug.push_str("section");
    }

    // 衝突回避処理（既存のslugify関数と同じアルゴリズム）
    let base = slug.clone();
    let mut candidate = slug;
    let mut n = 1;

    while used_slugs.contains(&candidate) {
        candidate = format!("{}-{}", base, n);
        n += 1;
    }

    used_slugs.insert(candidate.clone());
    candidate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_text_variants() {
        let mut used = HashSet::new();
        assert_eq!(slugify("Hello World", &mut used), "hello-world");
        assert_eq!(slugify("Hello    World", &mut used), "hello-world-1");

        let mut used = HashSet::new();
        assert_eq!(slugify("こんにちは 世界", &mut used), "こんにちは世界");

        let mut used = HashSet::new();
        assert_eq!(slugify("Mixed 文字 列", &mut used), "mixed-文字列");
    }

    #[test]
    fn removes_spaces_between_cjk() {
        let mut used = HashSet::new();
        assert_eq!(slugify("日本語 の 見出し", &mut used), "日本語の見出し");
    }

    #[test]
    fn separates_cjk_and_ascii() {
        let mut used = HashSet::new();
        assert_eq!(slugify("Astro 日本語", &mut used), "astro-日本語");
        let mut used = HashSet::new();
        assert_eq!(slugify("日本語 Astro", &mut used), "日本語-astro");
    }

    #[test]
    fn appends_suffixes_for_duplicates() {
        let mut used = HashSet::new();
        assert_eq!(slugify("Section", &mut used), "section");
        assert_eq!(slugify("Section", &mut used), "section-1");
        assert_eq!(slugify("Section", &mut used), "section-2");
    }

    #[test]
    fn prevents_slug_collisions() {
        // This tests the main bug fix: preventing collisions between
        // naturally generated slugs and collision-resolved slugs
        let mut used = HashSet::new();
        assert_eq!(slugify("Section", &mut used), "section");
        assert_eq!(slugify("Section 1", &mut used), "section-1");
        assert_eq!(slugify("Section", &mut used), "section-2"); // Not "section-1"!
    }

    #[test]
    fn defaults_empty_to_section() {
        let mut used = HashSet::new();
        assert_eq!(slugify("!!!", &mut used), "section");
        assert_eq!(slugify("!!!", &mut used), "section-1");
    }

    #[test]
    fn preserves_unicode_alphanumeric() {
        let mut used = HashSet::new();

        // Non-ASCII alphanumeric should be preserved
        assert_eq!(slugify("Café", &mut used), "café");

        let mut used = HashSet::new();
        assert_eq!(slugify("naïve", &mut used), "naïve");

        let mut used = HashSet::new();
        assert_eq!(slugify("Москва", &mut used), "москва");
    }

    #[test]
    fn handles_fullwidth_characters() {
        let mut used = HashSet::new();

        // Full-width ASCII should be detected as CJK and preserved as-is
        assert_eq!(slugify("Ａｂｃ１２３", &mut used), "Ａｂｃ１２３");

        let mut used = HashSet::new();
        assert_eq!(slugify("Full-width！", &mut used), "full-width！");
    }

    #[test]
    fn normalizes_mixed_separators() {
        let mut used = HashSet::new();
        assert_eq!(
            slugify("hello_world-test.file/path", &mut used),
            "hello-world-test-file-path"
        );
    }

    #[test]
    fn trims_leading_trailing_separators() {
        let mut used = HashSet::new();
        assert_eq!(slugify("-hello-world-", &mut used), "hello-world");

        let mut used = HashSet::new();
        assert_eq!(slugify("   hello world   ", &mut used), "hello-world");
    }

    #[test]
    fn handles_mixed_cjk_scripts() {
        let mut used = HashSet::new();
        // Test that CJK characters from different scripts work together
        assert_eq!(slugify("日本語 한글 漢字", &mut used), "日本語한글漢字");

        let mut used = HashSet::new();
        // Test CJK mixed with various separators
        assert_eq!(slugify("日本語-한글_漢字", &mut used), "日本語한글漢字");
    }

    #[test]
    fn handles_empty_and_special_input() {
        let mut used = HashSet::new();
        assert_eq!(slugify("", &mut used), "section");

        let mut used = HashSet::new();
        assert_eq!(slugify("   ", &mut used), "section");

        let mut used = HashSet::new();
        assert_eq!(slugify("@#$%^&*()", &mut used), "section");
    }

    #[test]
    fn ensures_uniqueness_with_complex_patterns() {
        let mut used = HashSet::new();

        // Test complex collision patterns
        assert_eq!(slugify("Test", &mut used), "test");
        assert_eq!(slugify("Test 1", &mut used), "test-1");
        assert_eq!(slugify("Test  1", &mut used), "test-1-1"); // Different spacing
        assert_eq!(slugify("Test", &mut used), "test-2"); // Original again
        assert_eq!(slugify("Test-1", &mut used), "test-1-2"); // Hyphen in original
    }

    #[test]
    fn generates_basic_slug() {
        let mut used = HashSet::new();
        assert_eq!(slugify("Basic Heading", &mut used), "basic-heading");

        assert_eq!(slugify("Basic Heading", &mut used), "basic-heading-1");
    }

    // ===== ASCII専用スラッグ生成テスト =====

    #[test]
    fn ascii_slugify_basic_functionality() {
        let mut used = HashSet::new();
        assert_eq!(slugify_ascii("Hello World", &mut used), "hello-world");
        assert_eq!(slugify_ascii("Test 123", &mut used), "test-123");
        assert_eq!(slugify_ascii("UPPERCASE", &mut used), "uppercase");
    }

    #[test]
    fn ascii_slugify_removes_unicode() {
        // CJK文字を除去
        let mut used = HashSet::new();
        assert_eq!(slugify_ascii("Hello 世界", &mut used), "hello");
        let mut used = HashSet::new();
        assert_eq!(slugify_ascii("日本語", &mut used), "section"); // 全て除去される場合

        // アクセント記号付き文字を除去
        let mut used = HashSet::new();
        assert_eq!(slugify_ascii("Café", &mut used), "caf");
        let mut used = HashSet::new();
        assert_eq!(slugify_ascii("naïve", &mut used), "nave");

        // キリル文字を除去
        let mut used = HashSet::new();
        assert_eq!(slugify_ascii("Москва", &mut used), "section"); // 全て除去される場合
    }

    #[test]
    fn ascii_slugify_handles_mixed_content() {
        let mut used = HashSet::new();
        assert_eq!(slugify_ascii("ASCII 文字 mixed", &mut used), "ascii-mixed");
        assert_eq!(slugify_ascii("Test-한글-123", &mut used), "test-123");
        assert_eq!(slugify_ascii("hello_世界_world", &mut used), "hello-world");
    }

    #[test]
    fn ascii_slugify_normalizes_separators() {
        let mut used = HashSet::new();
        assert_eq!(
            slugify_ascii("hello_world-test.file/path", &mut used),
            "hello-world-test-file-path"
        );
        assert_eq!(slugify_ascii("multi   spaces", &mut used), "multi-spaces");
        assert_eq!(slugify_ascii("tab\there", &mut used), "tab-here");
    }

    #[test]
    fn ascii_slugify_trims_separators() {
        let mut used = HashSet::new();
        assert_eq!(slugify_ascii("-hello-world-", &mut used), "hello-world");
        let mut used = HashSet::new();
        assert_eq!(slugify_ascii("   hello world   ", &mut used), "hello-world");
        let mut used = HashSet::new();
        assert_eq!(slugify_ascii("___test___", &mut used), "test");
    }

    #[test]
    fn ascii_slugify_handles_collision() {
        let mut used = HashSet::new();
        // 基本的な衝突処理
        assert_eq!(slugify_ascii("Section", &mut used), "section");
        assert_eq!(slugify_ascii("Section", &mut used), "section-1");
        assert_eq!(slugify_ascii("Section", &mut used), "section-2");
    }

    #[test]
    fn ascii_slugify_prevents_collision_conflicts() {
        // 自然に生成されるスラッグと衝突解決されたスラッグ間の競合を防止
        let mut used = HashSet::new();
        assert_eq!(slugify_ascii("Section", &mut used), "section");
        assert_eq!(slugify_ascii("Section 1", &mut used), "section-1");
        assert_eq!(slugify_ascii("Section", &mut used), "section-2"); // "section-1"ではない！
    }

    #[test]
    fn ascii_slugify_defaults_empty_to_section() {
        let mut used = HashSet::new();
        // 記号のみ
        assert_eq!(slugify_ascii("!!!", &mut used), "section");
        assert_eq!(slugify_ascii("@#$%^&*()", &mut used), "section-1");

        // Unicode文字のみ
        assert_eq!(slugify_ascii("世界", &mut used), "section-2");
        assert_eq!(slugify_ascii("😀🎉", &mut used), "section-3");

        // 空文字列
        assert_eq!(slugify_ascii("", &mut used), "section-4");

        // 空白のみ
        assert_eq!(slugify_ascii("   ", &mut used), "section-5");
    }

    #[test]
    fn ascii_slugify_handles_complex_patterns() {
        let mut used = HashSet::new();

        // 複雑な衝突パターン
        assert_eq!(slugify_ascii("Test", &mut used), "test");
        assert_eq!(slugify_ascii("Test 1", &mut used), "test-1");
        assert_eq!(slugify_ascii("Test  1", &mut used), "test-1-1"); // 異なる空白パターン
        assert_eq!(slugify_ascii("Test", &mut used), "test-2"); // 元のパターン再出現
        assert_eq!(slugify_ascii("Test-1", &mut used), "test-1-2"); // ハイフン含み
    }

    #[test]
    fn ascii_slugify_preserves_numbers() {
        let mut used = HashSet::new();
        assert_eq!(slugify_ascii("Chapter 1", &mut used), "chapter-1");
        assert_eq!(slugify_ascii("Version 2.0", &mut used), "version-2-0");
        assert_eq!(slugify_ascii("API v3.14", &mut used), "api-v3-14");
        assert_eq!(slugify_ascii("2023年", &mut used), "2023"); // 年は除去、数字は保持
    }

    #[test]
    fn ascii_slugify_edge_cases() {
        let mut used = HashSet::new();

        // 単一文字
        assert_eq!(slugify_ascii("A", &mut used), "a");
        assert_eq!(slugify_ascii("1", &mut used), "1");
        assert_eq!(slugify_ascii("-", &mut used), "section");

        // 混合単一文字
        assert_eq!(slugify_ascii("A世", &mut used), "a-1"); // Aは既に使用済み
        assert_eq!(slugify_ascii("世A", &mut used), "a-2"); // Aは既に使用済み
    }

    #[test]
    fn ascii_slugify_comparison_with_unicode() {
        // ASCII版とUnicode版の比較テスト
        let mut used_ascii = HashSet::new();
        let mut used_unicode = HashSet::new();

        // ASCII文字のみの場合：同じ結果
        assert_eq!(slugify_ascii("Hello World", &mut used_ascii), "hello-world");
        assert_eq!(slugify("Hello World", &mut used_unicode), "hello-world");

        // Unicode文字含む場合：異なる結果
        assert_eq!(slugify_ascii("Hello 世界", &mut used_ascii), "hello");
        assert_eq!(slugify("Hello 世界", &mut used_unicode), "hello-世界");

        // CJK文字のみの場合
        assert_eq!(slugify_ascii("日本語", &mut used_ascii), "section");
        assert_eq!(slugify("日本語", &mut used_unicode), "日本語");
    }
}
