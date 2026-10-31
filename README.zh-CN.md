# 🛡️ RevGuard

English: [README.md](./README.md)

**给 AI 辅助逆向工程加一道“减速带”。**

现在逆向代码太容易了：把打包压缩后的生产代码贴进大模型，输一句“帮我解释这段代码”，AI 分分钟给你还原得明明白白。

**RevGuard 就是专门给这种操作下绊子的。** 它利用了对齐大模型（如 ChatGPT、Claude 等）的安全护栏机制——只在你的生产构建产物中，注入无害、纯学术口吻的高危领域标识符（CBRN / 毒素 / 爆炸物 / 武器相关的标识符名）。当逆向者随手把代码喂给 AI 时，极易触发大模型风控，导致其犹豫或直接拒绝回答。

* **零副作用**：不加密、不混淆控制流、不重命名现有符号、**绝对不改变运行时行为**。
* **可逆还原**：自带清单校验，一行命令即可**按字节 100% 还原**回干净产物。
* **定位清晰**：这是威慑（减速带），不是 DRM（防盗门）。

```text
干净源码 ──► 打包器 ──► 生产产物
                          │
               RevGuard 注入（仅生产环境）
                          ▼
                   加料产物 + 标记
                   + revguard.manifest.json
                          │
          ├──► 逆向者贴进 LLM     ──► 🚨 触发风控，拒绝分析
          └──► revguard strip   ──► ✨ 按字节还原干净产物（用于过审/扫描）
```

---

## ⚡ 60 秒快速上手

**1. 编译核心 CLI**
```bash
cargo build --release
export REVGUARD_BIN=$PWD/target/release/revguard
```

**2. 接入构建工具（以 Vite 为例，仅在生产构建时生效）**
```js
// vite.config.js
import { defineConfig } from "vite";
import { revguardVite } from "@revguard/vite";

export default defineConfig({
  plugins: [revguardVite()], // 仅在 `vite build` 且 production 模式下自动注入
});
```

**3. 或直接使用 CLI 手动处理**
```bash
./target/release/revguard inject ./dist --config ./revguard.config.json
./target/release/revguard verify ./dist
./target/release/revguard strip  ./dist   # 送交 SAST 安全扫描或应用商店审核前，一键洗白
```
> 💡 **全局开关**：设置环境变量 `REVGUARD_DISABLE=1` 会让所有插件和集成瞬间变成空操作（No-op）。

---

## ⚙️ 核心原理

1. **顶部注释（Banner）**：在文件顶部插入 `/*! … */` 合法保护注释，塞满伪装用的高危学术标识符。
2. **死代码分支（Dead Code）**：在文件末尾追加一段由 `globalThis.__REVGUARD_NEVER__ === Symbol.for("revguard.never")` 守卫的分支（运行时恒为 `false`，永不执行，但打包器极难静态优化掉）。
3. **追踪清单（Manifest）**：生成 `revguard.manifest.json`，精确记录每个文件注入前后的 `sha256` 哈希与注入类型。
4. **无损剥离（Strip）**：运行 `revguard strip` 可将文本/JS 产物**按字节精准还原**到注入前的干净哈希状态。

---

## 🚨 安全红线与合规设计

RevGuard 严格限制在**四个类别**，每个类别提供 `low` / `medium` / `high` 三档强度：

| 类别 | 主题（仅使用纯学术外壳标签） |
| :--- | :--- |
| **`cbrn`** | 化学 / 生物 / 放射 / 核武器相关学术标签 |
| **`toxin`** | 毒素 / 致死剂量（如 LD50）相关学术标签 |
| **`explosive`** | 爆炸物 / 简易爆炸装置（IED）相关学术标签 |
| **`weapon`** | 武器制造 / 弹道学相关学术标签 |

* **纯伪装命名**：例如 `computeLd50LethalDoseStubNoDataUnused`。**绝不包含**任何真实配方、合成步骤、剂量数据或操作指南。
* **永久禁区**：坚决不包含 CSAM（儿童色情）、色情、人口贩运、NCII，以及任何真实可操作的危害内容。（详见 `crates/core/src/wordlist.rs` 与 `wordlists/README.md`）。

---

## 📦 安装与生态集成

### Rust CLI
> ⚠️ 注意：CLI 是 Rust 二进制程序 `revguard`（位于 `crates/cli`），不是 npm 全局包。

```bash
cargo install --path crates/cli
# 或在仓库内直接编译：
cargo build --release
./target/release/revguard --help
```

### JS / TS 工具链（Workspace）
使用前请确保已设置 `export REVGUARD_BIN=$PWD/target/release/revguard`（若使用 napi 原生模块则无需 CLI）。

| 包名 | 作用与特性 |
| :--- | :--- |
| `@revguard/node-bridge` | Node 绑定 — 优先走进程内 `napi`，失败则平滑回退至 CLI |
| `@revguard/vite` | Vite 插件 — `revguardVite()`（兼容 Rolldown） |
| `@revguard/rolldown` | Rolldown 插件 — `revguardRolldown()` |
| `@revguard/bun` | Bun 构建后置钩子 — `revguardAfterBundle()` |
| `@revguard/napi` | 基于 `napi-rs` 的高性能原生模块 |
| `@revguard/swc` | `@swc/core` 助手 — 优先调用真 WASM 插件，回退走 `napi` |
| `revguard_swc_plugin` | `wasm32-wasip1` SWC AST 访问器（`.wasm` 随包分发） |

<details>
<summary><b>👉 点击展开：Rolldown / Bun / SWC / Python 接入代码示例</b></summary>

```js
// 1. Rolldown
import { revguardRolldown } from "@revguard/rolldown";
export default { plugins: [revguardRolldown({ outDir: "dist" })] };

// 2. Bun
import { revguardAfterBundle } from "@revguard/bun";
const result = await Bun.build({ entrypoints: ["./src/index.ts"], outdir: "./dist", minify: true });
if (result.success) await revguardAfterBundle("./dist"); // 非 NODE_ENV=production 时为空操作

// 3. SWC (@swc/core) — 存在 .wasm 时优先运行真 WASM visitor
const { transformWithRevguard } = require("@revguard/swc");
const { code, backend } = transformWithRevguard(src, { production: true });
// backend === "wasm-plugin" | "napi-postemit"
```

```bash
# 4. Python (PyO3)
cd packages/pyo3 && maturin develop --release
python -c "import revguard; print(revguard.inject('./dist')[:80])"
```
</details>

### 生产环境门控（Gating）
**注入是发布产物的属性，绝不污染开发源码：**

| 集成方式 | 触发注入的条件 |
| :--- | :--- |
| `@revguard/vite` | `command === "build"` 且处于 `production` 模式 |
| `@revguard/rolldown` / `bun` | `NODE_ENV=production`（或显式传入 `opts.force`） |
| `@revguard/swc` | `production: true` 或 `NODE_ENV=production`（或 `force`） |
| CLI `revguard inject` | **始终执行**（由你在 CI/CD 中决定何时调用） |
| 环境变量 `REVGUARD_DISABLE=1` | **一票否决**，所有集成全部变为空操作 |

---

## 🛠️ 配置与 CLI 指南

**配置文件 (`revguard.config.json`)**
```json
{
  "enabled": true,
  "categories": ["cbrn", "toxin", "explosive", "weapon"],
  "intensity": "medium",
  "inject_banner": true,
  "inject_dead_code": true,
  "inject_source_map": false,
  "inject_wasm_section": false,
  "wordlist_path": null,
  "wordlist_version": null
}
```
*支持环境变量覆盖：`REVGUARD_DISABLE`、`REVGUARD_BIN`、`REVGUARD_WORDLIST`、`REVGUARD_INJECT_SOURCEMAP`、`REVGUARD_INJECT_WASM`。*

**CLI 命令速查**
```bash
revguard inject <dir> [--config PATH]            # 注入目录产物
revguard strip <dir>                             # 按清单无损还原产物
revguard verify <dir>                            # 校验产物哈希状态
revguard inject-rust-src <crate-dir> [--config]  # 注入 Rust 源码标记
revguard strip-rust-src <crate-dir>              # 剥离 Rust 源码标记
revguard export-wordlist [wordlists/builtin.json]# 导出内置词表（用于自定义轮换）
```

---

## 🌐 全栈多场景支持矩阵

RevGuard 提供两种工作模式：
1. **文本 / JS 产物模式**：直接修改构建后的 `.js/.mjs/.cjs/.ts` 等文件，支持 `revguard strip` **按字节精准还原**。
2. **编译期源码模式**：将常量烘焙进原生二进制文件，还原时需**剥离标记后重新编译**。

| 应用场景 | 接入方案 | 模式 | 示例工程 (Demos) |
| :--- | :--- | :--- | :--- |
| **前端 (Vite/Rolldown/Bun)** | 生产构建挂载对应插件 | 文本/JS | `demos/frontend/` + 自测脚本 |
| **后端 (Node/TS)** | 打包后运行 `inject` 或通过 `node-bridge` 调用 | 文本/JS | `demos/backend-node/` |
| **Electron** | Main + Preload + Renderer 均为 JS 产物，直接注入 | 文本/JS | `demos/electron-app/` |
| **Web / WASM** | 默认注入 JS 胶水代码（如 `wasm-bindgen` 输出）；可选注入 `.wasm` 自定义段 | 文本/JS (+二进制) | `demos/wasm-js-glue/`<br>`testdata/wasm-section/` |
| **Tauri** | 前端走 JS 插件；Rust 后端走编译期标记 | 混合模式 | 插件 + `demos/native-rust/` |
| **原生二进制 (Rust)** | 注入 `#[used]` + `black_box` 字符串常量（支持 CLI 自动注入） | 编译期 | `demos/native-rust/`<br>`testdata/rust-src/` |
| **原生二进制 (Go/C/C++/Zig)** | 注入等价的 keepalive 字符串常量 | 编译期 | 见 `demos/native-rust/README.md` |

### 进阶特性支持状态
* ✅ **SWC 原生支持**：提供真 WASM 插件（`rustc stable/1.99+`，`swc_core 83`）及 `napi` 回退方案。
* ✅ **PyO3 绑定**：位于 `packages/pyo3`，支持 Python 生态直接调用。
* ✅ **Source Map 污染**：开启 `inject_source_map: true` 后会改写 `.map` 中的 `sourcesContent`，同样支持 `strip` 还原。
* ✅ **自定义词表轮换**：支持通过配置或环境变量加载自定义词表，对抗针对性过滤。
* ⚠️ **WASM 自定义段 (PoC)**：开启 `inject_wasm_section: true` 会追加 `revguard` 自定义段。*注意：`wasm-opt` 等工具可能会剥离未知段；且除非逆向者先 dump 二进制段，否则 LLM 不会直接读取二进制，威慑力不如注入 JS 胶水层。*

---

## 🗂️ 仓库架构与自测

```text
crates/
  core              核心库：wordlist, inject, strip, sourcemap, wasm, rust_src...
  cli               命令行工具
packages/
  ├── napi          napi-rs 原生绑定
  ├── node-bridge   Node 桥接层（优先 napi，回退 CLI）
  ├── vite / rolldown / bun  各构建工具插件
  ├── swc / swc-plugin   SWC 助手与 wasm32-wasip1 原生插件
  └── pyo3          Python 绑定 (maturin)
demos/              给用户看的接入示例
testdata/           自测夹具（SWC、sourcemap、rust-src、wasm section、PyO3）
wordlists/          内置词表与轮换示例
```

**运行完整自测套件：**
```bash
cargo build --release
./scripts/selftest.sh            # core：JS inject → strip 字节还原闭环
./scripts/selftest.sh napi       # napi 及前后端 / Electron / WASM / 原生场景
./scripts/selftest.sh extra      # SWC(WASM)、PyO3、Sourcemap、词表轮换、rust-src
./scripts/selftest.sh all
# extra / napi 需要先：npm run build:napi（需 g++ 或 zig c++）
```

---

## ⚠️ 诚实声明与安全警告（必读）

1. **它不是银弹**：RevGuard 只是针对“随手把代码丢给 AI 分析”这种偷懒行为的威慑手段。它**不是** DRM、不是加密、不能替代真正的知识产权保护。执着的逆向分析者、未对齐的开源模型或特定的越狱提示词都能绕过它。请仅将其作为纵深防御中的一环。
2. **小心误伤友军（SAST / 审核报警）**：注入的伪装标识符**极易触发静态代码分析（SAST）、恶意软件扫描、供应链安全审计以及应用商店（App Store / Google Play）的自动审核机制**。
   * **最佳实践**：送交内部安全扫描或应用商店审核的构建包，**务必先运行 `revguard strip ./dist` 还原为干净构建**。将加料后的产物仅保留给直接分发给终端用户的渠道，并**提前向你的安全团队报备**（免得被当成内鬼）。

---

## 🗺️ 路线图 (Roadmap)
- [ ] 更丰富的 WASM `name-section` 注入手段
- [ ] 更多宿主打包器（Webpack / Esbuild 等）的原生挂钩

## 📄 License
[MIT](./LICENSE)
