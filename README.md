# manga-rs

跨平台的漫画库，旨在提供一套通用的接口访问大量平台的漫画资源。

基于本库实现的工具：专注于命令行的 [manga-cli](https://github.com/Hentioe/manga-cli)，桌面版和手机版还在预备阶段。

## 支持平台

以下列表的状态跟踪 master 分支的最新实现，按照模块名排序：

| 名称           | 域名               | 支持状态 |          标签          |
| :------------- | :----------------- | :------: | :--------------------: |
| 動漫狂         | www.cartoonmad.com |   ⭕️    |          中文          |
| 动漫屋         | www.dm5.com        |   ⭕️    |          中文          |
| 动漫之家       | manhua.dmzj.com    |   ⭕️    |          中文          |
| E-Hentai       | e-hentai.org       |   ⭕️    | 英文, 日文, 中文, NSFW |
| 18H 宅宅愛動漫 | 18h.animezilla.com |   ⭕️    |       中文, NSFW       |
| 汗汗酷漫       | www.hhimm.com      |   ⭕️    |          中文          |
| KuKu 动漫      | comic.kukudm.com   |   ⭕️    |          中文          |
| LHScan         | lhscan.net         |   ⭕️    |          英文          |
| Manganelo      | manganelo.com      |   ⭕️    |          英文          |
| 漫画 DB        | www.manhuadb.com   |   ⭕️    |          中文          |
| 漫画堆         | www.manhuadui.com  |   ⭕️    |          中文          |
| 漫画柜         | www.manhuagui.com  |   ⭕️    |          中文          |
| 漫画人         | www.manhuaren.com  |   ⭕️    |          中文          |
| 177 漫畫       | www.177pic.info    |   ⭕️    |    中文, 日文, NSFW    |
| 青空漫画       | www.qkmh5.com      |    ❌    |          中文          |
| 非常爱漫       | comic.veryim.com   |    ❌    |          中文          |
| 新新漫画网     | www.177mh.net      |   ⭕️    |          中文          |
| YYLS 漫畫      | 8comic.se          |   ⭕️    |          中文          |

不支持你需要的平台？请在 [Issues](https://github.com/Hentioe/manga-rs/issues) 中告诉我。

## 文档说明

目前作者还没有为本项目编写文档。本库最终会以 C ABI 兼容的形式公开接口，并提供各种 bingding 以支持不同的操作系统。
