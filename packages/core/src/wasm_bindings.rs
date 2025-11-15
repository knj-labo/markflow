use crate::*;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

/// WASM用render関数
///
/// JavaScriptから呼び出し可能なMarkdownレンダリング関数。
/// 現在は基本的な機能を提供し、PR3でエラーハンドリングとパフォーマンスを改善予定。
#[wasm_bindgen]
pub fn render_wasm(source: String, options: JsValue) -> Result<JsValue, JsValue> {
    // JavaScriptから渡されたオプションパラメータを安全に処理する
    let opts = if options.is_null() || options.is_undefined() {
        Options::default()
    } else {
        from_value(options).unwrap_or_else(|_| Options::default())
    };

    // メインのMarkdownレンダリング処理を実行
    let result = render(&source, &opts); // Rustネイティブ関数を呼び出し

    // レンダリング結果をJavaScript側で使える形式に変換
    to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// WASM用slugify関数
///
/// 単体のテキストからslug（URL対応文字列）を生成する関数。
/// 衝突検出は行わないため、複数の見出しがある場合は`render_wasm`を使用してください。
#[wasm_bindgen]
pub fn slugify_wasm(text: String) -> String {
    // 衝突防止用のハッシュセットを空で初期化
    let mut used_slugs = std::collections::HashSet::new(); // まだ使用されたスラッグは無い状態

    // メインのスラッグ生成関数を呼び出し
    crate::slugify::slugify(&text, &mut used_slugs) // テキストを渡してURL対応文字列に変換
}
