# mikack

跨平台的图集检索库（以漫画为主），旨在提供一套通用的接口访问不同来源的在线资源。

## 周边生态

本项目仅仅是一个库，对其它操作系统和编程语言的支持请使用 FFI 库和其衍生的各种绑定。如果你是普通用户，请跳转至已开发完成的工具。

### FFI 和衍生库

- [mikack-ffi](https://github.com/Hentioe/mikack-ffi)（外部接口，以 C ABI 为主）
- [mikack-dart](https://github.com/Hentioe/mikack-dart)（Dart 语言实现的绑定）

### 基于本库实现的工具

- [mikack-cli](https://github.com/Hentioe/mikack-cli)（使用 Rust 的 native 程序，适用于命令行）
- [mikack-mobile](https://github.com/Hentioe/mikack-mobile)（使用 Flutter，适用于手机，**推荐**）
- mikack-desktop（使用 Electron，适用于桌面系统）
- [mikack-favicon](https://github.com/Hentioe/mikack-favicon)（给客户端提供统一的平台图标）

_其中 mikack-cli 项目由于技术同构的原因直接使用了 Rust 接口，参考性较低。而桌面版的开发计划还未启动。_

## 支持平台

以下列表的状态跟踪 master 分支的最新实现，按照模块名排序：

| 名称             | 域名                                                   | 可阅读？ | 可搜索？ |          标签          |
| :--------------- | :----------------------------------------------------- | :------: | :------: | :--------------------: |
| 壁咚漫画         | www<i>.</i>bidongmh<i>.</i>com                         |    ✓     |    ✓     |       中文, NSFW       |
| 百年漫画         | [www.bnmanhua.com](https://www.bnmanhua.com)           |    ✓     |    ✓     |          中文          |
| 動漫狂           | [www.cartoonmad.com](https://www.cartoonmad.com)       |    ✓     |    ✓     |          中文          |
| comico           | [www.comico.com.tw](http://www.comico.com.tw)          |    ✓     |    ✓     |          中文          |
| 动漫屋（漫画人） | [www.dm5.com](https://www.dm5.com)                     |    ✓     |    ✓     |          中文          |
| 动漫之家         | [manhua.dmzj.com](https://manhua.dmzj.com)             |    ✓     |    ✓     |          中文          |
| E-Hentai         | e-hentai<i>.</i>org                                    |    ✓     |    ✓     | 英文, 日文, 中文, NSFW |
| 18H 宅宅愛動漫   | 18h<i>.</i>animezilla<i>.</i>com                       |    ✓     |          |       中文, NSFW       |
| 古风漫画网       | [www.gufengmh8.com](https://www.gufengmh8.com)         |    ✓     |    ✓     |          中文          |
| 喵绅士           | c-upp<i>.</i>com                                       |    ✓     |    ✓     | 英文, 日文, 中文, NSFW |
| 汗汗酷漫         | [www.hhimm.com](http://www.hhimm.com)                  |    ✓     |    ✓     |          中文          |
| KuKu 动漫        | [comic.ikkdm.com](http://comic.kkkkdm.com)             |    ✓     |    ✓     |          中文          |
| 快看漫画         | [www.kuaikanmanhua.com](https://www.kuaikanmanhua.com) |    ✓     |    ✓     |          中文          |
| LoveHeaven       | [loveheaven.net](https://loveheaven.net)               |    ✓     |    ✓     |          英文          |
| Luscious         | www<i>.</i>luscious<i>.</i>net                         |    ✓     |    ✓     | 英文, 日文, 中文, NSFW |
| Mangabz          | [www.mangabz.com](http://www.mangabz.com)              |    ✓     |    ✓     |          中文          |
| Manganelo        | [manganelo.com](https://manganelo.com)                 |    ✓     |    ✓     |          英文          |
| Mangareader      | [www.mangareader.net](http://www.mangareader.net)      |          |    ✓     |          英文          |
| 漫画 DB          | [www.manhuadb.com](https://www.manhuadb.com)           |    ✓     |    ✓     |          中文          |
| 漫画堆           | [www.manhuadui.com](https://www.manhuadui.com)         |    ✓     |    ✓     |          中文          |
| 漫画柜           | [www.manhuagui.com](https://www.manhuagui.com)         |    ✓     |    ✓     |          中文          |
| 漫画铺           | [www.manhuapu.com](http://www.manhuapu.com)            |    ✓     |    ✓     |          中文          |
| nhentai          | nhentai<i>.</i>net                                     |    ✓     |    ✓     | 英文, 日文, 中文, NSFW |
| 9hentai          | 9hentai<i>.</i>com                                     |    ✓     |    ✓     |       英文, NSFW       |
| 90 漫画网        | [www.90mh.com](http://www.90mh.com)                    |    ✓     |    ✓     |          中文          |
| 177 漫畫         | www<i>.</i>177pic<i>.</i>info                          |    ✓     |    ✓     |    中文, 日文, NSFW    |
| ONE 漫画         | [www.onemanhua.com](https://www.onemanhua.com)         |    ✓     |    ✓     |          中文          |
| 扑飞漫画         | [www.pufei8.com](http://www.pufei8.com)                |    ✓     |    ✓     |          中文          |
| 奇妙漫画         | [www.qimiaomh.com](https://www.qimiaomh.com)           |    ✓     |    ✓     |          中文          |
| 土豪漫画         | [www.tohomh123.com](https://www.tohomh123.com)         |    ✓     |    ✓     |          中文          |
| 台灣成人 H 漫    | twhentai<i>.</i>com                                    |    ✓     |    ✓     |    中文, 日文, NSFW    |
| 二次元動漫       | [www.2animx.com](http://www.2animx.com)                |    ✓     |    ✓     |          中文          |
| 紳士漫畫         | www<i>.</i>wnacg<i>.</i>org                            |    ✓     |    ✓     |       中文, NSFW       |
| 57 漫画网        | [www.wuqimh.com](http://www.wuqimh.com/)               |    ✓     |    ✓     |          中文          |
| 新新漫画网       | [www.177mh.net](https://www.177mh.net)                 |    ✓     |    ✓     |          中文          |
| YYLS 漫畫        | [8comic.se](https://8comic.se)                         |    ✓     |    ✓     |          中文          |

不支持你想看的平台？请在 [Issues](https://github.com/Hentioe/mikack/issues) 中告诉我。
