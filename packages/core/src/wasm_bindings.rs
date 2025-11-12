use crate::*;
use wasm_bindgen::prelude::*;

/// WASM用render関数
///
/// JavaScriptから呼び出し可能なMarkdownレンダリング関数。
/// 現在は基本的な機能を提供し、PR3でエラーハンドリングとパフォーマンスを改善予定。
#[wasm_bindgen]
pub fn render_wasm(source: String, options: JsValue) -> Result<JsValue, JsValue> {
    // JavaScriptから渡されたオプションパラメータを安全に処理する
    let opts = if options.is_null() || options.is_undefined() {
        // JavaScriptでnullまたはundefinedが渡された場合
        Options::default() // Rustのデフォルト設定を使用
    } else {
        // JavaScriptオブジェクトが渡された場合の変換処理
        options
            .into_serde() // JavaScriptオブジェクト → Rust構造体への変換を試行
            .unwrap_or_else(|_| Options::default()) // 変換失敗時はデフォルト設定で安全にフォールバック
    };

    // メインのMarkdownレンダリング処理を実行
    let result = render(&source, &opts); // Rustネイティブ関数を呼び出し

    // レンダリング結果をJavaScript側で使える形式に変換
    JsValue::from_serde(&result) // Rust構造体 → JavaScriptオブジェクトへのシリアライズ
        .map_err(|e| JsValue::from_str(&e.to_string())) // エラー発生時は文字列メッセージに変換してJavaScriptに返す
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
