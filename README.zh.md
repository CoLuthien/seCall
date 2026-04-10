<div align="center">

# seCall

搜索你与AI助手的所有对话。

[![Rust](https://img.shields.io/badge/Rust-1.75+-f74c00?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![SQLite](https://img.shields.io/badge/SQLite-FTS5-003B57?logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![MCP](https://img.shields.io/badge/MCP-Protocol-5A67D8)](https://modelcontextprotocol.io/)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE)

<br/>

[**`한국어`**](README.md) · [**`English`**](README.md#en) · [**`日本語`**](README.ja.md) · **`中文`**

</div>

---

## 什么是 seCall？

seCall 是一个面向 AI 代理会话的本地优先搜索引擎。它收集 **Claude Code**、**Codex CLI**、**Gemini CLI**、**claude.ai**、**ChatGPT** 的对话日志，通过 BM25 + 向量混合搜索建立索引，并通过 CLI/MCP 服务器/Obsidian 兼容知识库提供服务。

与 AI 的对话是知识资产。seCall 让它们变得可搜索、可浏览、相互关联。

## 主要功能

### 多代理收集

| 代理 | 格式 | 状态 |
|---|---|---|
| Claude Code | JSONL | ✅ 稳定 |
| Codex CLI | JSONL | ✅ 稳定 |
| Gemini CLI | JSON | ✅ 稳定 |
| claude.ai | JSON (ZIP) | ✅ v0.2 新增 |
| ChatGPT | JSON (ZIP) | ✅ v0.2.3 新增 |

### 混合搜索

- **BM25 全文搜索**: SQLite FTS5 + 韩语形态素分析（[Lindera](https://github.com/lindera/lindera) ko-dic / [Kiwi-rs](https://github.com/bab2min/kiwi) 可选）
- **向量语义搜索**: [Ollama](https://ollama.com/) BGE-M3 嵌入（1024维）+ **HNSW ANN 索引**（[usearch](https://github.com/unum-cloud/usearch)）
- **Reciprocal Rank Fusion (RRF)**: BM25/向量独立执行后融合（k=60）+ 会话级多样性
- **LLM 查询扩展**: 通过 Claude Code 进行自然语言查询扩展

### 知识库

Obsidian 兼容 Markdown 知识库（双层结构）:

```
vault/
├── raw/sessions/    # 不可变会话原始数据
│   └── YYYY-MM-DD/  # 按日期整理
├── wiki/            # AI 生成的知识页面
│   ├── projects/    # 项目摘要
│   ├── topics/      # 技术主题页
│   └── decisions/   # 架构决策记录
└── graph/           # Knowledge Graph 输出
    └── graph.json   # 节点/边数据
```

### Knowledge Graph

确定性地提取会话间关系来构建知识图谱（无需 LLM 调用）:

- **节点类型**: session, project, agent, tool — 从 frontmatter 自动提取
- **边类型**: `belongs_to`, `by_agent`, `uses_tool`, `same_project`, `same_day`
- **增量构建**: 仅添加新会话节点，关系边全量重新计算以保证准确性
- **MCP 工具**: `graph_query` — AI 代理可探索会话间关系（BFS，最多3跳）

### MCP 服务器

```bash
# stdio 模式（Claude Code, Cursor 等）
secall mcp

# HTTP 模式
secall mcp --http 127.0.0.1:8080
```

提供工具: `recall`, `get`, `status`, `wiki_search`, `graph_query`

### 多设备知识库同步

```bash
# 完整同步: git pull → reindex → ingest → wiki → graph → git push
secall sync

# 仅本地（跳过 git）
secall sync --local-only
```

## 快速开始

### 前置条件

- Rust 1.75+（从源码构建时）
- Claude Code, Codex CLI, Gemini CLI 至少一个
- [Ollama](https://ollama.com/) — 用于向量搜索（可选，没有则仅用 BM25）
- **Windows**: MSVC 工具链

### Step 1. 安装

```bash
git clone https://github.com/hang-in/seCall.git
cd seCall
cargo install --path crates/secall
```

预编译二进制文件可从 [Releases](https://github.com/hang-in/seCall/releases) 下载。

### Step 2. 初始化

```bash
# 交互式引导（推荐）
secall init

# 或直接指定参数
secall init --vault ~/Documents/Obsidian\ Vault/seCall
```

不带参数运行 `secall init` 将启动交互式向导:
- 知识库路径设置
- Git 远程仓库（可选）
- 分词器选择（lindera/kiwi）
- 嵌入后端选择（ollama/none）
- Ollama 安装检查 + 自动拉取 `bge-m3` 模型

### Step 3. 收集会话

```bash
# 自动检测 Claude Code 会话
secall ingest --auto

# 或一键全量同步
secall sync
```

### Step 4. 搜索

```bash
# BM25 全文搜索
secall recall "BM25 索引实现"

# 向量语义搜索（需要 Ollama）
secall recall "搜索管道如何工作" --vec
```

## 配置

```bash
# 查看当前配置
secall config show

# 修改配置
secall config set output.timezone Asia/Shanghai
secall config set search.tokenizer lindera
secall config set embedding.backend ollama
```

| 键 | 说明 | 默认值 |
|---|---|---|
| `vault.path` | Obsidian 知识库路径 | `~/obsidian-vault/seCall` |
| `vault.branch` | Git 分支名 | `main` |
| `search.tokenizer` | 分词器（`lindera` / `kiwi`） | `lindera` |
| `embedding.backend` | 嵌入后端（`ollama` / `ort` / `none`） | `ollama` |
| `output.timezone` | 时区（IANA） | `UTC` |

## CLI 参考

| 命令 | 说明 |
|---|---|
| `secall init` | 交互式引导 |
| `secall ingest [path] --auto` | 收集会话 |
| `secall sync` | 完整同步 |
| `secall recall <query>` | 混合搜索 |
| `secall get <id> [--full]` | 查看会话详情 |
| `secall status` | 索引统计 |
| `secall embed [--all]` | 生成向量嵌入 |
| `secall config show\|set\|path` | 查看/修改配置 |
| `secall graph build\|stats\|export` | Knowledge Graph 管理 |
| `secall wiki update\|status` | Wiki 生成/状态 |
| `secall mcp [--http <addr>]` | 启动 MCP 服务器 |
| `secall lint` | 一致性验证 |

## MCP 集成

在 Claude Code 配置（`~/.claude/settings.json`）中添加:

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

## 技术栈

| 分类 | 技术 |
|---|---|
| 语言 | Rust 1.75+ |
| 数据库 | SQLite + FTS5 |
| NLP | Lindera ko-dic + Kiwi-rs (macOS/Linux) |
| 嵌入 | Ollama BGE-M3 (1024维) / ONNX Runtime |
| MCP 服务器 | rmcp (stdio + HTTP) |
| 知识库 | Obsidian 兼容 Markdown |

## 参考

- **[LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)** (Andrej Karpathy) — 用 LLM 逐步构建知识库的模式
- **[qmd](https://github.com/tobi/qmd)** (Tobi Lütke) — Markdown 本地搜索引擎
- **[graphify](https://github.com/safishamsi/graphify)** (Safi Shamsi) — 从文件夹构建知识图谱

使用 [tunaFlow](https://github.com/hang-in/tunaFlow) 多代理工作流平台开发。

## 许可证

[AGPL-3.0](LICENSE)
