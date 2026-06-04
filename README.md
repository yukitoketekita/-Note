# ヰnote

<!-- markdownlint-disable -->

<div align="center">

<img src="./src-tauri/icons/icon.png" width="120" alt="ヰnote 图标">

# ヰnote

轻量、本地、随叫随到的便签工具<br>

基于 Tauri 2 + React 构建

[反馈问题](https://github.com/TsukiraiSaigiaochi/-Note/issues) · [更新日志](https://github.com/TsukiraiSaigiaochi/-Note/releases)

[![Version](https://img.shields.io/github/v/release/TsukiraiSaigiaochi/-Note)](https://github.com/TsukiraiSaigiaochi/-Note/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
![Stars](https://img.shields.io/github/stars/TsukiraiSaigiaochi/-Note?color=ffcb47&labelColor=black)</br>

![React 19](https://img.shields.io/badge/React-19-blue?logo=react)
![Tauri v2](https://img.shields.io/badge/Tauri-v2-%2324C8D8?logo=tauri)
![Rust Edition 2021](https://img.shields.io/badge/Rust-2021-%23000000?logo=rust)<br>

</div>

<!-- markdownlint-restore -->

---

## 创作动机

我一直想找一个用着顺手、随叫随到的简单笔记软件。

不需要复杂的知识库，不需要沉重的工作流，只要能在需要的时候立刻打开，写下几句话，然后安静地待在桌面上就够了。

正好我又是个ヰ组，当然也希望写笔记的时候能看着我推写。

于是，在一个不想复习期末考试的下午，它诞生了。

## 功能特点

- **Markdown 编辑与预览** — 支持 Markdown 语法，用更接近写作的方式记录想法

  ![主窗口截图](Docs/Images/main-window.png)

- **快捷便签** — 通过托盘或全局快捷键随时唤出便签窗口

  ![快捷便签示例](Docs/images/quick-note.gif)

- **本地存储** — 笔记内容保存在本地，不依赖云端服务

- **导入 Markdown** — 支持 `.md` 文件导入

## 应用场景

- **桌子上的草稿本怎么用，它就怎么用**

- 写代码的时候随手记一下思路

- 打游戏的时候随手记一下注意事项

- 看资料、听课、复习时快速记录片段想法

- 给老师、领导、朋友，甚至 AI 发送消息前，先在这里编辑一遍

- 当作临时剪贴板，暂存需要反复复制的文本

## 下载安装

前往 [GitHub Releases](https://github.com/TsukiraiSaigiaochi/-Note/releases) 下载最新版本。



## 从源码构建

### 环境要求

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri CLI 2](https://tauri.app/)

### 步骤

```bash
git clone https://github.com/TsukiraiSaigiaochi/-Note.git

cd -Note

npm install

# 开发模式
npm run tauri dev

# 构建发布版本
npm run tauri build