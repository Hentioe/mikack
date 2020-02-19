# mikack

跨平台的漫画库，旨在提供一套通用的接口访问不同来源的在线资源。

_本项目曾经临时命名为 manga-rs_

## 周边项目

本项目仅仅是一个库，以 C ABI 兼容的形式公开接口，支持多种操作系统和编程语言。

### 基于本库实现的工具：

- [mikack-cli](https://github.com/Hentioe/mikack-cli)（使用 Rust 的 native 程序，适用于命令行）
- mikack-mobile（使用 Flutter 或 React Native，适用于手机）
- mikack-desktop（使用 Electron，适用于桌面系统）

注意，mikack-cli 项目由于技术同构的原因直接使用了 Rust 接口，参考性较低。

**手机版和桌面版还在预备开发中。**

注意：本项目占比很高的 Javascript 代码作用于某些平台资源的加解密，和具体实现/接口无关。所以没有 JS 能直接访问的接口，需要作为 native 模块通过 binding 使用。

## 支持平台

以下列表的状态跟踪 master 分支的最新实现，按照模块名排序：

| 名称             | 域名                                                   | 支持状态 |          标签          |
| :--------------- | :----------------------------------------------------- | :------: | :--------------------: |
| 壁咚漫画         | www<i>.</i>bidongmh<i>.</i>com                         |   ⭕️    |       中文, NSFW       |
| 百年漫画         | [www.bnmanhua.com](https://www.bnmanhua.com)           |   ⭕️    |          中文          |
| 動漫狂           | [www.cartoonmad.com](https://www.cartoonmad.com)       |   ⭕️    |          中文          |
| comico           | [www.comico.com.tw](http://www.comico.com.tw)          |   ⭕️    |          中文          |
| 动漫屋（漫画人） | [www.dm5.com](https://www.dm5.com)                     |   ⭕️    |          中文          |
| 动漫之家         | [manhua.dmzj.com](https://manhua.dmzj.com)             |   ⭕️    |          中文          |
| E-Hentai         | e-hentai<i>.</i>org                                    |   ⭕️    | 英文, 日文, 中文, NSFW |
| 18H 宅宅愛動漫   | 18h<i>.</i>animezilla<i>.</i>com                       |   ⭕️    |       中文, NSFW       |
| 古风漫画网       | [www.gufengmh8.com](https://www.gufengmh8.com)         |   ⭕️    |          中文          |
| 喵绅士           | c-upp<i>.</i>com                                       |   ⭕️    | 英文, 日文, 中文, NSFW |
| 汗汗酷漫         | [www.hhimm.com](http://www.hhimm.com)                  |   ⭕️    |          中文          |
| 扑飞漫画         | [www.ipufei.com](http://www.ipufei.com)                |   ⭕️    |          中文          |
| 快看漫画         | [www.kuaikanmanhua.com](https://www.kuaikanmanhua.com) |   ⭕️    |          中文          |
| KuKu 动漫        | [comic.kukudm.com](https://comic.kukudm.com)           |   ⭕️    |          中文          |
| LoveHeaven       | [loveheaven.net](https://loveheaven.net)               |   ⭕️    |          英文          |
| Luscious         | www<i>.</i>luscious<i>.</i>net                         |   ⭕️    | 英文, 日文, 中文, NSFW |
| Mangabz          | [www.mangabz.com](http://www.mangabz.com)              |   ⭕️    |          中文          |
| Manganelo        | [manganelo.com](https://manganelo.com)                 |   ⭕️    |          英文          |
| 漫画 DB          | [www.manhuadb.com](https://www.manhuadb.com)           |   ⭕️    |          中文          |
| 漫画堆           | [www.manhuadui.com](https://www.manhuadui.com)         |   ⭕️    |          中文          |
| 漫画柜           | [www.manhuagui.com](https://www.manhuagui.com)         |   ⭕️    |          中文          |
| 漫画铺           | [www.manhuapu.com](http://www.manhuapu.com)            |   ⭕️    |          中文          |
| nhentai          | nhentai<i>.</i>net                                     |   ⭕️    | 英文, 日文, 中文, NSFW |
| 9hentai          | 9hentai<i>.</i>com                                     |   ⭕️    |       英文, NSFW       |
| 177 漫畫         | www<i>.</i>177pic<i>.</i>info                          |   ⭕️    |    中文, 日文, NSFW    |
| 奇妙漫画         | [www.qimiaomh.com](https://www.qimiaomh.com)           |   ⭕️    |          中文          |
| 土豪漫画         | [www.tohomh123.com](https://www.tohomh123.com)         |   ⭕️    |          中文          |
| 二次元動漫       | [www.2animx.com](http://www.2animx.com)                |   ⭕️    |          中文          |
| 紳士漫畫         | www<i>.</i>wnacg<i>.</i>org                            |   ⭕️    |       中文, NSFW       |
| 新新漫画网       | [www.177mh.net](https://www.177mh.net)                 |   ⭕️    |          中文          |
| YYLS 漫畫        | [8comic.se](https://8comic.se)                         |   ⭕️    |          中文          |

不支持你需要的平台？请在 [Issues](https://github.com/Hentioe/mikack/issues) 中告诉我。

## 文档说明

### 一些概念

本库将漫画类网站的资源抽象为了三个最基本的通用模型：

| 结构名  | 含义 | 备注                     |
| :------ | :--- | :----------------------- |
| Comic   | 漫画 | 漫画主页数据             |
| Chapter | 章节 | 如常见单位：话/集        |
| Page    | 页   | 如 1P、2P、3P 中的单个 P |

简单的基本关系：漫画包含多个章节，章节包含多个页。

三个基本模型的结构：

| 所属结构 | 字段名         | 备注                            |
| -------: | :------------- | ------------------------------- |
|    Comic | `title`        | 漫画名称                        |
|    Comic | `url`          | 漫画链接                        |
|    Comic | `cover`        | 漫画封面                        |
|    Comic | `chapters`     | 章节列表                        |
|  Chapter | `title`        | 章节名称                        |
|  Chapter | `url`          | 章节链接                        |
|  Chapter | `which`        | 章节索引                        |
|  Chapter | `pages`        | 页面列表                        |
|  Chapter | `page_headers` | 下载页面资源必要的 HTTP headers |
|     Page | `n`            | 页码，当前以 0 开始             |
|     Page | `address`      | 资源文件地址                    |
|     Page | `fname`        | 资源文件名称                    |
|     Page | `fmime`        | 资源文件的 MIME                 |

您可能会注意到并没有与“平台”相关的模型，因为平台被抽象为了具有相同行为的组件，这类组件无法自行创建，需要从其它 API 中获取。

这类定义了平台行为的组件被称作 Extracotr（提取器），它本至上是一个个 Rust 中的 Trait 对象。

### 基本 API

#### 获取平台列表：

```rust
use mikack::extractors;

for (domain, name) in extractors::platforms().iter() {
    domain  // => 平台域名
    name    // => 平台名称
}
```

`extractors::platforms()` 函数返回个包含所有平台基本信息的 HashMap 结构，键是域名，值是平台的名称。

其中域名是获取平台对应的 Extractor 实例的必要参数，不过对于其它 API 调用而言，名称都是无用的（一般用于 UI 显示）。

#### 过滤平台列表：

```rust
use mikack::{extractors, models::Tag};

// 过滤平台列表
// 参数一：包含的标签
// 参数二：不包含的标签
extractors::find_platforms(&[Tag::Chinese], &[Tag::NSFW]); // => 非 NSFW 的中文平台列表
```

使用标签过滤平台列表，标签来自：

```rust
use mikack::models::Tag;

let tags = Tag::all();  // => 全部标签
tags[0].to_string();    // => 标签名称
```

#### 获取指定的 Extractor

```rust
use mikack::extractors;

let domain = "www.example.com";
let extractor = extractors::get_extr(domain).expect(&format!("Unsupported platform {}", domain));
```

通过域名获取指定的 Extractor 实例，若没有找到则返回 None。与示例代码不同，域名参数应该来源于上一个 API 的返回值，而非自行输入。

#### 获取漫画列表

```rust
let page = 1;
let comics = extractor.index(page)?;

comics; // => 漫画列表
```

通过 Extractor 实例的 `index` 方法获取到的漫画列表一般是平台最近更新的内容。您还可以通过 `is_pageable` 方法兼容不同平台的分页支持情况：

```rust
if page > 1 && !extractor.is_pageable() {
    // 此平台不支持分页，没有下一页
}
```

目前绝大多数平台都支持分页，不支持分页的情况一般是平台一次性返回了所有内容。

#### 搜索漫画

```rust
let keywords = "海贼王";
let comics = extractor.search(keywords)?;

comics // => 漫画列表
```

类似的，您可以通过 `is_searchable` 方法兼容不同平台的搜索支持情况：

```rust
if !extractor.is_searchable() {
    // 此平台不支持搜索
}
```

当前只有极少数平台不支持搜索，在不支持平台的 Extractor 对象上调用 `search` 方法将始终返回空列表。

#### 获取漫画章节

```rust
let mut comic = Comic::from_link("进击的巨人", "https://www.manhuadui.com/manhua/jinjidejuren/");
extractor.fetch_chapters(comic)?;

comic.chapters // => 章节列表
```

调用 `fetch_chapters` 需要将一个可变的 `Comic` 对象作为参数，章节列表会填充到 `chapters` 属性。参数中的 Comic 对象只需要有效的 url 参数。

#### 获取页面资源

```rust
let mut chapter = Chapter::from_link(
    "进击的巨人- 126话 骄傲",
    "https://www.manhuadui.com/manhua/jinjidejuren/459779.html"
);
let iter = extractor.pages_iter(chapter)?;
for page in iter {                      // 遍历页面资源
    let page = page?;                   // => 页面资源
    let address = page.address.clone(); // => 资源地址
}
```

当我们在开发在线阅读的客户端时，如果一次性获取全部地址才开始阅读，会导致过长的等待的时间。假设某漫画某章有 100 页，可能需要提前加载完 100 个页面才能返回全部页面资源，显然是不明智的。

使用迭代器手动控制翻页，逻辑将是这个样子：

```rust
// 开始载入章节
let iter = extractor.pages_iter(..)?;   // 创建一个页面迭代器
// 需要注意的是，创建迭代器也需要载入页面，但通常只解析一个页面
// 在创建迭代器的同时已经将除资源地址以外的章节数据载入好了
let title = iter.chapter_title_clone(); // => 从迭代器获取章节标题
let total = iter.total;                 // => 从迭代器获取页面数量
// 翻页
if let Some(page) = iter.next()? {
    // 加载当前页
} else {
    // 没有下一页啦
}
```

迭代器 API 是懒加载的，适合在线阅读用途的客户端应用。值得一提的是 [mikack-cli](https://github.com/Hentioe/mikack-cli) 项目也使用迭代器 API，因为它要显示加载进度。所以实际上大多数场景都推荐使用迭代器 API。

如果您不在意资源获取的延迟问题，也可以使用更简单的 `fetch_pages` 方法。它将页面列表填充到参数 `chapter` 对象中，此处不做演示。

_文档还在紧张撰写中……_
