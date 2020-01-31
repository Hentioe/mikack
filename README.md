# manga-rs

跨平台的漫画库，旨在提供一套通用的接口访问不同来源的在线资源。

## 周边项目

本项目仅仅是一个库，以 C ABI 兼容的形式公开接口，支持多种操作系统和编程语言。

### 基于本库实现的工具：

- [manga-cli](https://github.com/Hentioe/manga-cli)（使用 Rust 的 native 程序，适用于命令行）
- manga-mobile（使用 Flutter 或 React Native，适用于手机）
- manga-desktop（使用 Electron，适用于桌面系统）

注意，manga-cli 项目由于技术同构的原因直接使用了 Rust 接口，参考性较低。

**手机版和桌面版还在预备开发中。**

注意：本项目是纯 Rust 实现，Javascript 代码是辅助某些平台加解密所用。所以也并不存在 JS 直接访问的接口，需要作为 native 模块通过 binding 使用。

## 支持平台

以下列表的状态跟踪 master 分支的最新实现，按照模块名排序：

| 名称           | 域名                                     | 支持状态 |          标签          |
| :------------- | :--------------------------------------- | :------: | :--------------------: |
| 動漫狂         | [www.cartoonmad.com](www.cartoonmad.com) |   ⭕️    |          中文          |
| 动漫屋         | [www.dm5.com](www.dm5.com)               |   ⭕️    |          中文          |
| 动漫之家       | [manhua.dmzj.com](manhua.dmzj.com)       |   ⭕️    |          中文          |
| E-Hentai       | [e-hentai.org](e-hentai.org)             |   ⭕️    | 英文, 日文, 中文, NSFW |
| 18H 宅宅愛動漫 | [18h.animezilla.com](18h.animezilla.com) |   ⭕️    |       中文, NSFW       |
| 汗汗酷漫       | [www.hhimm.com](www.hhimm.com)           |   ⭕️    |          中文          |
| KuKu 动漫      | [comic.kukudm.com](comic.kukudm.com)     |   ⭕️    |          中文          |
| LHScan         | [lhscan.net](lhscan.net)                 |   ⭕️    |          英文          |
| Luscious       | [www.luscious.net](www.luscious.net)     |   ⭕️    | 英文, 日文, 中文, NSFW |
| Manganelo      | [manganelo.com](manganelo.com)           |   ⭕️    |          英文          |
| 漫画 DB        | [www.manhuadb.com](www.manhuadb.com)     |   ⭕️    |          中文          |
| 漫画堆         | [www.manhuadui.com](www.manhuadui.com)   |   ⭕️    |          中文          |
| 漫画柜         | [www.manhuagui.com](www.manhuagui.com)   |   ⭕️    |          中文          |
| 漫画人         | [www.manhuaren.com](www.manhuaren.com)   |   ⭕️    |          中文          |
| 177 漫畫       | [www.177pic.info](www.177pic.info)       |   ⭕️    |    中文, 日文, NSFW    |
| 青空漫画       | www.qkmh5.com                            |    ❌    |          中文          |
| 二次元動漫     | [www.2animx.com](www.2animx.com)         |   ⭕️    |          中文          |
| 非常爱漫       | comic.veryim.com                         |    ❌    |          中文          |
| 新新漫画网     | [www.177mh.net](www.177mh.net)           |   ⭕️    |          中文          |
| YYLS 漫畫      | [8comic.se](8comic.se)                   |   ⭕️    |          中文          |

不支持你需要的平台？请在 [Issues](https://github.com/Hentioe/manga-rs/issues) 中告诉我。

## 文档说明

### 一些概念

本库将漫画类网站的资源抽象为了三个最基本的通用模型，分别是：

1. Comic（漫画，指具体的漫画主页）
1. Chapter（章节，例如常见单位：话/集）
1. Page（页，即 1P、2P、3P 中的单个 P）

基本关系非常简单：漫画包含多个章节，章节包含多个页。以下是三个基本模型的字段说明：

- Comic
  - title（漫画名称）
  - url（漫画链接）
  - cover（漫画封面）
  - chapters（章节列表）
- Chapter
  - title（章节名称）
  - url（章节链接）
  - which（章节索引）
  - pages（页面列表）
  - page_headers（下载页面资源需包含的请求头）
- Page
  - n（页码，当前以 0 开始）
  - address（资源文件地址）
  - fname（资源文件名称）
  - fmime（资源文件的 MIME）

更多文档内容作者正在紧急撰写中……
