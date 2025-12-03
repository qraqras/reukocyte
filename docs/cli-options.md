# Reukocyte CLI Options

RuboCopとの互換性を考慮したCLIオプション一覧です。

## 実装予定オプション

### 優先度: 必須 (Phase 1)

| オプション | 短縮形 | 引数 | 説明 | RuboCop互換 |
|-----------|--------|------|------|-------------|
| `[files...]` | - | paths | チェック対象のファイル/ディレクトリ | ✅ |
| `--autocorrect` | `-a` | - | 安全な自動修正のみ適用 | ✅ |
| `--autocorrect-all` | `-A` | - | すべての自動修正を適用（unsafe含む） | ✅ |
| `--only` | - | `COP1,COP2,...` | 指定したcopのみ実行 | ✅ |
| `--except` | - | `COP1,COP2,...` | 指定したcopを除外 | ✅ |
| `--format` | `-f` | `FORMATTER` | 出力フォーマット指定 | ✅ |

### 優先度: 高 (Phase 2)

| オプション | 短縮形 | 引数 | 説明 | RuboCop互換 |
|-----------|--------|------|------|-------------|
| `--config` | `-c` | `FILE` | 設定ファイル指定 | ✅ |
| `--stdin` | `-s` | `FILE` | STDINから読み込み（エディタ連携用） | ✅ |
| `--fail-level` | - | `SEVERITY` | 終了コードに影響する最低レベル | ✅ |
| `--color` / `--no-color` | - | - | カラー出力の制御 | ✅ |

### 優先度: 中 (Phase 3)

| オプション | 短縮形 | 引数 | 説明 | RuboCop互換 |
|-----------|--------|------|------|-------------|
| `--lint` | `-l` | - | Lintカテゴリのみ実行 | ✅ |
| `--fix-layout` | `-x` | - | Layoutカテゴリのみ（自動修正付き） | ✅ |
| `--safe` | - | - | Safeなcopのみ実行 | ✅ |
| `--fail-fast` | `-F` | - | 最初のエラーで停止 | ✅ |
| `--force-exclusion` | - | - | 設定のExcludeを強制適用 | ✅ |
| `--display-cop-names` / `--no-display-cop-names` | `-D` | - | cop名を表示 | ✅ |

### 優先度: 低 (Phase 4)

| オプション | 短縮形 | 引数 | 説明 | RuboCop互換 |
|-----------|--------|------|------|-------------|
| `--version` | `-v` | - | バージョン表示 | ✅ |
| `--help` | `-h` | - | ヘルプ表示 | ✅ |
| `--debug` | `-d` | - | デバッグ情報表示 | ✅ |
| `--display-time` | - | - | 経過時間を表示 | ✅ |
| `--parallel` / `--no-parallel` | `-P` | - | 並列実行の制御 | ✅ |
| `--stderr` | - | - | すべてをstderrに出力 | ✅ |

---

## フォーマッター一覧

| フォーマッター | 短縮 | 説明 | 実装優先度 |
|---------------|------|------|-----------|
| `json` | `j` | JSON形式（RuboCop互換必須） | 必須 |
| `simple` | `s` | シンプルなテキスト出力 | 必須 |
| `quiet` | `q` | 最小限の出力 | 高 |
| `progress` | `p` | 進捗表示（RuboCopデフォルト） | 高 |
| `clang` | `c` | Clang風の出力 | 中 |
| `emacs` | `e` | Emacs形式 | 中 |
| `github` | `g` | GitHub Actions形式 | 中 |
| `files` | `fi` | ファイル名のみ | 低 |

---

## Severity レベル

| レベル | 短縮 | 説明 |
|--------|------|------|
| `info` | `I` | 情報 |
| `refactor` | `R` | リファクタリング推奨 |
| `convention` | `C` | 規約違反 |
| `warning` | `W` | 警告 |
| `error` | `E` | エラー |
| `fatal` | `F` | 致命的エラー |

---

## 終了コード

| コード | 意味 |
|--------|------|
| `0` | 成功（違反なし、または修正完了） |
| `1` | 違反あり |
| `2` | エラー（無効なオプション、ファイル不在等） |

---

## 実装しないオプション

以下はReukocyteの目的（RuboCopの前処理ツール）に不要なため実装しません：

| オプション | 理由 |
|-----------|------|
| `--server` 関連 | サーバモードは不要（十分高速） |
| `--lsp` | LSPはRuboCopに任せる |
| `--auto-gen-config` | 設定生成はRuboCopに任せる |
| `--cache` 関連 | キャッシュは将来検討 |
| `--require` | プラグイン機構は不要 |
| `--plugin` | プラグイン機構は不要 |
| `--show-cops` | RuboCopに任せる |
| `--init` | RuboCopに任せる |

---

## 使用例

```bash
# 基本的なチェック
rueko .

# 自動修正（safe）
rueko -a .

# 自動修正（すべて）
rueko -A .

# JSON出力
rueko -f json .

# 特定のcopのみ
rueko --only Layout/TrailingWhitespace,Lint/Debugger .

# 特定のcopを除外
rueko --except Lint/Debugger .

# Layoutのみ自動修正
rueko -x .

# エディタ連携（stdin）
cat file.rb | rueko --stdin file.rb

# 設定ファイル指定
rueko -c .rubocop.yml .

# 複数オプション
rueko -a -f json --only Layout/TrailingWhitespace src/
```

---

## 設定ファイル対応 (.rubocop.yml)

Reukocyteは以下の設定項目を読み取ります：

```yaml
AllCops:
  Include:
    - '**/*.rb'
  Exclude:
    - 'vendor/**/*'
    - 'tmp/**/*'

Layout/TrailingWhitespace:
  Enabled: true

Lint/Debugger:
  Enabled: true
```

※ RuboCopの設定ファイルをそのまま使用可能
