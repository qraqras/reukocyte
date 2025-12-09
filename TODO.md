# Reukocyte TODO

## Layout ルール実装 (優先順位順)

### Phase 1: 行ベース・シンプル
- [x] Layout/TrailingWhitespace - 行末の空白
- [x] Layout/TrailingEmptyLines - ファイル末尾の空行チェック
- [x] Layout/LeadingEmptyLines - ファイル先頭の不要な空行
- [x] Layout/EmptyLines - 連続する空行（2行以上）
- [x] Layout/IndentationStyle - タブ vs スペース

### Phase 2: インデント系
- [x] Layout/IndentationWidth - インデント幅（2スペース）

### Phase 3: スペース系（AST必要）
- [ ] Layout/SpaceAfterComma - カンマ後のスペース
- [ ] Layout/SpaceAroundOperators - 演算子周りのスペース
- [ ] Layout/SpaceInsideHashLiteralBraces - `{ key: value }` のスペース
- [ ] Layout/SpaceInsideBlockBraces - `{ |x| ... }` のスペース
- [ ] Layout/ExtraSpacing - 不要な連続スペース

## 実装メモ

### 各ルールの実装方針

#### TrailingEmptyLines
- EnforcedStyle: `final_newline` (デフォルト) / `final_blank_line`
- ファイル末尾の `\n` の数をチェック
- Fix: 適切な改行数に調整

#### LeadingEmptyLines
- ファイル先頭の空行を検出
- Fix: 先頭の空行を削除

#### EmptyLines
- 2行以上連続する空行を検出
- Fix: 1行に削減

#### IndentationStyle
- EnforcedStyle: `spaces` (デフォルト) / `tabs`
- 行頭のタブ/スペースをチェック
- Fix: 設定に応じて変換

#### IndentationWidth
- Width: 2 (デフォルト)
- インデントが2の倍数かチェック
- AST情報が必要（ブロック開始位置など）

### 共通パターン
```rust
pub fn check(checker: &mut Checker) {
    let edit_ranges = collect_edit_ranges(checker.source());
    for (start, end, replacement) in edit_ranges {
        let fix = Fix::safe(vec![Edit::replacement(start, end, replacement)]);
        checker.report(RULE_ID, message, severity, start, end, Some(fix));
    }
}
```
