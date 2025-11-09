/// HTMLサニタイズ
///
/// HTMLで特別な意味を持つ文字を安全なHTMLエンティティに変換し、
/// XSS攻撃やHTMLインジェクションを防ぎます。
/// CommonMark仕様に準拠した文字エスケープを行います。
///
/// # エスケープされる文字
///
/// | 文字 | エンティティ | 理由 |
/// |------|-------------|------|
/// | `&`  | `&amp;`     | HTMLエンティティの開始文字 |
/// | `"`  | `&quot;`    | HTML属性値の区切り文字 |
/// | `<`  | `&lt;`      | HTMLタグの開始文字 |
/// | `>`  | `&gt;`      | HTMLタグの終了文字 |
/// | `'`  | `&#39;`     | HTML属性値の区切り文字（シングルクォート） |
///
/// # 参考実装
///
/// markdown-rsのencode関数を参考に実装：
/// <https://github.com/wooorm/markdown-rs/blob/main/src/util/encode.rs>
pub fn sanitize_html(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut result = String::with_capacity(s.len());
    let mut start = 0;

    for (index, &byte) in bytes.iter().enumerate() {
        let replacement = match byte {
            b'&' => "&amp;",
            b'"' => "&quot;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'\'' => "&#39;",
            _ => continue,
        };

        // 前回の位置から現在位置までの文字を追加
        result.push_str(&s[start..index]);
        // 特殊文字を置換
        result.push_str(replacement);
        // 次の開始位置を更新
        start = index + 1;
    }

    // 特殊文字が一つもない場合は元の文字列をそのまま返す
    if start == 0 {
        return s.to_string();
    }

    // 残りの文字を追加
    result.push_str(&s[start..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_basic_characters() {
        // 基本的なHTML特殊文字のエスケープ
        assert_eq!(sanitize_html("&"), "&amp;");
        assert_eq!(sanitize_html("<"), "&lt;");
        assert_eq!(sanitize_html(">"), "&gt;");
        assert_eq!(sanitize_html("\""), "&quot;");
        assert_eq!(sanitize_html("'"), "&#39;");
    }

    #[test]
    fn sanitize_mixed_content() {
        // 混在コンテンツのテスト
        assert_eq!(
            sanitize_html("Hello <world> & \"friends\"!"),
            "Hello &lt;world&gt; &amp; &quot;friends&quot;!"
        );
        assert_eq!(
            sanitize_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;"
        );
    }

    #[test]
    fn handle_normal_text() {
        // 特殊文字が含まれない場合
        let normal_text = "Hello world 123 こんにちは";
        assert_eq!(sanitize_html(normal_text), normal_text);

        // 空文字列
        assert_eq!(sanitize_html(""), "");
    }

    #[test]
    fn escape_special_characters() {
        // 特殊文字のみ
        assert_eq!(sanitize_html("&<>\"'"), "&amp;&lt;&gt;&quot;&#39;");
        assert_eq!(sanitize_html("&&&"), "&amp;&amp;&amp;");
    }

    #[test]
    fn preserve_unicode_safety() {
        // Unicode文字と特殊文字の混在
        assert_eq!(
            sanitize_html("日本語 & English < 中文 > \"text\""),
            "日本語 &amp; English &lt; 中文 &gt; &quot;text&quot;"
        );
        assert_eq!(
            sanitize_html("🚀 <rocket> & 'emoji'"),
            "🚀 &lt;rocket&gt; &amp; &#39;emoji&#39;"
        );
    }

    #[test]
    fn sanitize_attribute_values() {
        // HTML属性でよく使われるパターン
        assert_eq!(
            sanitize_html("class=\"container\" data-value='test'"),
            "class=&quot;container&quot; data-value=&#39;test&#39;"
        );
    }
}
