# Tarkov Interactive Map Assistant - 运行与打包指南

## 环境要求

- **Node.js**: 18+ 或 20+ (推荐)
- **Rust**: 最新稳定版 (通过 rustup 安装)
- **WebView2**: Windows 10/11 (Windows 11 自带，Windows 10 可能需要安装)
- **包管理器**: npm (Node.js 自带)

### 检查环境

```bash
# 检查 Node.js 版本
node -v

# 检查 Rust 版本
rustc --version
cargo --version
```

### 安装 Rust (如果未安装)

访问 https://rustup.rs 下载安装，或运行：

```bash
# Windows
winget install Rustlang.Rustup
```

---

## 安装依赖

### 首次安装

```bash
# 清理旧依赖（如果存在）
rm -rf node_modules package-lock.json

# 安装依赖（使用 legacy-peer-deps 解决版本冲突）
npm install --legacy-peer-deps
```

### 常规安装

```bash
npm install --legacy-peer-deps
```

---

## 开发运行

### 启动开发服务器

```bash
# 启动 Tauri 开发模式（包含前端热重载）
npm run tauri:dev
```

### 仅启动前端开发服务器

```bash
npm run dev
```

访问：http://localhost:8001

---

## 生产打包

### 完整打包流程

```bash
# 1. 构建前端
npm run build

# 2. 打包 Tauri 应用
npm run tauri:build
```

### 一键打包（推荐）

```bash
npm run build && npm run tauri:build
```

### 打包产物位置

- **可执行文件**: `src-tauri\target\release\tarkov-interactive-map-assistant-desktop.exe`
- **MSI 安装包**: `src-tauri\target\release\bundle\msi\*.msi`
- **NSIS 安装程序**: `src-tauri\target\release\bundle\nsis\*.exe`

---

## 网络代理设置（如果打包时下载失败）

### 设置代理

```bash
# 设置代理（替换 10808 为你的代理端口）
set HTTPS_PROXY=http://127.0.0.1:10808
set HTTP_PROXY=http://127.0.0.1:10808
set ALL_PROXY=http://127.0.0.1:10808

# 然后运行打包命令
npm run tauri:build
```

### 取消代理

```bash
set HTTPS_PROXY=
set HTTP_PROXY=
set ALL_PROXY=
```

或直接关闭终端重新打开。

---

## 代码检查与格式化

```bash
# 代码检查
npm run lint

# 自动修复代码问题
npm run lint:fix

# 代码格式化
npm run prettier

# 完整修复（格式化 + 检查修复）
npm run fix
```

---

## 常见问题

### 1. PowerShell 禁止运行脚本

**错误**: `无法加载文件 npm.ps1，因为在此系统上禁止运行脚本`

**解决**:
```powershell
# 以管理员身份打开 PowerShell，运行：
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### 2. npm install 依赖冲突

**错误**: `ERESOLVE could not resolve`

**解决**:
```bash
npm install --legacy-peer-deps
```

### 3. rolldown 原生绑定缺失

**错误**: `Cannot find native binding`

**解决**:
```bash
rm -rf node_modules package-lock.json
npm install --legacy-peer-deps
```

### 4. 打包时网络超时

**错误**: `timeout: global` 或 `Peer disconnected`

**解决**: 设置代理（见上方"网络代理设置"）

### 5. 前端资源未找到

**错误**: `Unable to find your web assets`

**解决**:
```bash
# 先构建前端
npm run build

# 再打包
npm run tauri:build
```

---

## WebView2 配置

当前配置：`embedBootstrapper`（自动下载安装 WebView2）

### 修改为依赖系统 WebView2

编辑 `src-tauri/tauri.conf.json`：

```json
"windows": {
  "webviewInstallMode": {
    "type": "skip"
  }
}
```

### 修改为完全离线打包（内置 WebView2）

```json
"windows": {
  "webviewInstallMode": {
    "type": "offlineInstaller"
  }
}
```

注意：离线打包会增加约 150MB 安装包体积。

---

## 游戏路径配置

应用首次运行需要配置：

1. **截图目录**（默认位置）:
   ```
   C:\Users\{用户名}\Documents\Escape from Tarkov\Screenshots
   ```

2. **游戏根目录**（默认位置）:
   ```
   C:\Battlestate Games\EFT
   ```

在应用设置面板中选择对应目录即可。

---

## 项目结构

```
tarkov-interactive-map-assistant-desktop-main/
├── src/                    # 前端源码 (React + TypeScript)
├── src-tauri/              # Rust 后端源码
├── dist/                   # 前端构建产物
├── node_modules/           # Node.js 依赖
├── package.json            # 项目配置
├── vite.config.ts          # Vite 配置
├── tsconfig.json           # TypeScript 配置
└── CLAUDE.md               # Claude Code 工作规范
```

---

## 技术支持

- **项目仓库**: https://github.com/TomyTang331/tarkov-interactive-map-assistant-desktop
- **Tauri 文档**: https://tauri.app/
- **问题反馈**: 提交 GitHub Issues
