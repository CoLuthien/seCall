<div align="center">

# seCall

AIエージェントとのすべての会話を検索しましょう。

[![Rust](https://img.shields.io/badge/Rust-1.75+-f74c00?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![SQLite](https://img.shields.io/badge/SQLite-FTS5-003B57?logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![MCP](https://img.shields.io/badge/MCP-Protocol-5A67D8)](https://modelcontextprotocol.io/)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE)

<br/>

[**`한국어`**](README.md) · [**`English`**](README.md#en) · **`日本語`** · [**`中文`**](README.zh.md)

</div>

---

## seCallとは？

seCallはAIエージェントセッション向けのローカルファースト検索エンジンです。**Claude Code**、**Codex CLI**、**Gemini CLI**、**claude.ai**、**ChatGPT**の会話ログを収集し、BM25＋ベクトルハイブリッド検索でインデックスを作成し、CLI/MCPサーバー/Obsidian互換ナレッジボールトとして提供します。

AIとの会話はナレッジ資産です。seCallはそれを検索可能で、探索可能で、相互に接続された形にします。

## 主な機能

### マルチエージェント収集

| エージェント | フォーマット | 状態 |
|---|---|---|
| Claude Code | JSONL | ✅ 安定版 |
| Codex CLI | JSONL | ✅ 安定版 |
| Gemini CLI | JSON | ✅ 安定版 |
| claude.ai | JSON (ZIP) | ✅ v0.2 新規 |
| ChatGPT | JSON (ZIP) | ✅ v0.2.3 新規 |

### ハイブリッド検索

- **BM25全文検索**: SQLite FTS5 + 韓国語形態素解析 ([Lindera](https://github.com/lindera/lindera) ko-dic / [Kiwi-rs](https://github.com/bab2min/kiwi) 選択可能)
- **ベクトル意味検索**: [Ollama](https://ollama.com/) BGE-M3エンベディング (1024次元) + **HNSW ANNインデックス** ([usearch](https://github.com/unum-cloud/usearch))
- **Reciprocal Rank Fusion (RRF)**: BM25/ベクトル独立実行後に結合 (k=60) + セッションレベル多様性
- **LLMクエリ拡張**: Claude Codeによる自然言語クエリ拡張

### ナレッジボールト

Obsidian互換マークダウンボールト（2層構造）:

```
vault/
├── raw/sessions/    # 不変セッション原本
│   └── YYYY-MM-DD/  # 日付別整理
├── wiki/            # AI生成ナレッジページ
│   ├── projects/    # プロジェクト別サマリー
│   ├── topics/      # 技術トピックページ
│   └── decisions/   # アーキテクチャ意思決定記録
└── graph/           # Knowledge Graph出力
    └── graph.json   # ノード/エッジデータ
```

### Knowledge Graph

セッション間の関係を決定的に抽出してナレッジグラフを構築します（LLM呼び出し不要）:

- **ノードタイプ**: session, project, agent, tool — frontmatterから自動抽出
- **エッジタイプ**: `belongs_to`, `by_agent`, `uses_tool`, `same_project`, `same_day`
- **増分ビルド**: 新規セッションのみノード追加、関係エッジは全体再計算
- **MCPツール**: `graph_query` — AIエージェントがセッション間関係を探索（BFS、最大3ホップ）

### MCPサーバー

```bash
# stdioモード (Claude Code, Cursor等)
secall mcp

# HTTPモード
secall mcp --http 127.0.0.1:8080
```

提供ツール: `recall`, `get`, `status`, `wiki_search`, `graph_query`

### マルチデバイスボールト同期

```bash
# 完全同期: git pull → reindex → ingest → wiki → graph → git push
secall sync

# ローカルのみ（git省略）
secall sync --local-only
```

## クイックスタート

### 前提条件

- Rust 1.75+（ソースビルド時）
- Claude Code, Codex CLI, Gemini CLI のいずれか
- [Ollama](https://ollama.com/) — ベクトル検索用（オプション、なければBM25のみ）
- **Windows**: MSVCツールチェーン

### Step 1. インストール

```bash
git clone https://github.com/hang-in/seCall.git
cd seCall
cargo install --path crates/secall
```

ビルド済みバイナリは [Releases](https://github.com/hang-in/seCall/releases) からダウンロードできます。

### Step 2. 初期化

```bash
# 対話式オンボーディング（推奨）
secall init

# または引数を直接指定
secall init --vault ~/Documents/Obsidian\ Vault/seCall
```

`secall init`を引数なしで実行すると対話式ウィザードが起動します:
- ボールトパス設定
- Gitリモート（オプション）
- トークナイザー選択 (lindera/kiwi)
- エンベディングバックエンド選択 (ollama/none)
- Ollamaインストール確認 + `bge-m3`モデル自動pull

### Step 3. セッション収集

```bash
# Claude Codeセッション自動検出
secall ingest --auto

# または一括同期
secall sync
```

### Step 4. 検索

```bash
# BM25全文検索
secall recall "BM25インデキシング実装"

# ベクトル意味検索（Ollama必要）
secall recall "検索パイプラインの仕組み" --vec
```

## 設定

```bash
# 現在の設定を確認
secall config show

# 設定変更
secall config set output.timezone Asia/Tokyo
secall config set search.tokenizer lindera
secall config set embedding.backend ollama
```

| キー | 説明 | デフォルト |
|---|---|---|
| `vault.path` | Obsidianボールトパス | `~/obsidian-vault/seCall` |
| `vault.branch` | Gitブランチ名 | `main` |
| `search.tokenizer` | トークナイザー (`lindera` / `kiwi`) | `lindera` |
| `embedding.backend` | エンベディング (`ollama` / `ort` / `none`) | `ollama` |
| `output.timezone` | タイムゾーン (IANA) | `UTC` |

## CLIリファレンス

| コマンド | 説明 |
|---|---|
| `secall init` | 対話式オンボーディング |
| `secall ingest [path] --auto` | セッション収集 |
| `secall sync` | 完全同期 |
| `secall recall <query>` | ハイブリッド検索 |
| `secall get <id> [--full]` | セッション詳細表示 |
| `secall status` | インデックス統計 |
| `secall embed [--all]` | ベクトルエンベディング生成 |
| `secall config show\|set\|path` | 設定の確認/変更 |
| `secall graph build\|stats\|export` | Knowledge Graph管理 |
| `secall wiki update\|status` | Wiki生成/状態確認 |
| `secall mcp [--http <addr>]` | MCPサーバー起動 |
| `secall lint` | 整合性検証 |

## MCP連携

Claude Code設定（`~/.claude/settings.json`）に追加:

```json
{
  "mcpServers": {
    "secall": {
      "command": "secall",
      "args": ["mcp"]
    }
  }
}
```

## 技術スタック

| 分類 | 技術 |
|---|---|
| 言語 | Rust 1.75+ |
| データベース | SQLite + FTS5 |
| NLP | Lindera ko-dic + Kiwi-rs (macOS/Linux) |
| エンベディング | Ollama BGE-M3 (1024次元) / ONNX Runtime |
| MCPサーバー | rmcp (stdio + HTTP) |
| ボールト | Obsidian互換 Markdown |

## 出典

- **[LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)** (Andrej Karpathy) — LLMでナレッジベースを段階的に構築するパターン
- **[qmd](https://github.com/tobi/qmd)** (Tobi Lütke) — マークダウン用ローカル検索エンジン
- **[graphify](https://github.com/safishamsi/graphify)** (Safi Shamsi) — ファイルフォルダからナレッジグラフを構築

[tunaFlow](https://github.com/hang-in/tunaFlow)マルチエージェントワークフロープラットフォームで開発されました。

## ライセンス

[AGPL-3.0](LICENSE)
