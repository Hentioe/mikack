# mikack

A cross-platform atlas retrieval library (mainly comics), designed to provide a common set of interfaces to access online resources from different sources.

## Surrounding ecology

This project is only a library, for the support of other operating systems and programming languages, please use the FFI library and its various bindings. If you are a normal user, jump to the tools that have been developed.

### FFI and derivative libraries

- [mikack-ffi](https://github.com/Hentioe/mikack-ffi) (External interface, mainly C ABI)
- [mikack-dart](https://github.com/Hentioe/mikack-dart) (Dart language binding)

### Tools based on this library

- [mikack-cli](https://github.com/Hentioe/mikack-cli) (Native program written in Rust, suitable for command line)
- [mikack-mobile](https://github.com/Hentioe/mikack-mobile) (Use Flutter framework, suitable for mobile phones. **recommend**)
- mikack-desktop (Use Electron framework, suitable for desktop systems)
- [mikack-favicon](https://github.com/Hentioe/mikack-favicon) (Tools for generating platform icons)

_Among them, the mikack-cli project directly uses the Rust interface, which is less referential due to the technical isomorphism. And development plans for the desktop version have not yet started._

## Source support

The status of the following list tracks the latest implementation of the master branch, sorted by module name:

| NAME             | DOMAIN                                                 | READABLE? | SEARCHABLE? |               TAGS               |
| :--------------- | :----------------------------------------------------- | :-------: | :---------: | :------------------------------: |
| 百年漫画         | [www.bnmanhua.com](https://www.bnmanhua.com)           |     ✓     |      ✓      |             Chinese              |
| 動漫狂           | [www.cartoonmad.com](https://www.cartoonmad.com)       |     ✓     |      ✓      |             Chinese              |
| comico           | [www.comico.com.tw](http://www.comico.com.tw)          |     ✓     |      ✓      |             Chinese              |
| 动漫屋（漫画人） | [www.dm5.com](https://www.dm5.com)                     |     ✓     |      ✓      |             Chinese              |
| 动漫之家         | [manhua.dmzj.com](https://manhua.dmzj.com)             |     ✓     |      ✓      |             Chinese              |
| E-Hentai         | e-hentai<i>.</i>org                                    |     ✓     |      ✓      | English, Japanese, Chinese, NSFW |
| 18H 宅宅愛動漫   | 18h<i>.</i>animezilla<i>.</i>com                       |     ✓     |             |          Chinese, NSFW           |
| 古风漫画网       | [www.gufengmh8.com](https://www.gufengmh8.com)         |     ✓     |      ✓      |             Chinese              |
| 喵绅士           | c-upp<i>.</i>com                                       |     ✓     |      ✓      | English, Japanese, Chinese, NSFW |
| 汗汗酷漫         | [www.hhimm.com](http://www.hhimm.com)                  |     ✓     |      ✓      |             Chinese              |
| KuKu 动漫        | [comic.ikkdm.com](http://comic.kkkkdm.com)             |     ✓     |      ✓      |             Chinese              |
| 快看漫画         | [www.kuaikanmanhua.com](https://www.kuaikanmanhua.com) |     ✓     |      ✓      |             Chinese              |
| LoveHeaven       | [loveheaven.net](https://loveheaven.net)               |     ✓     |      ✓      |             English              |
| Luscious         | www<i>.</i>luscious<i>.</i>net                         |     ✓     |      ✓      | English, Japanese, Chinese, NSFW |
| Mangabz          | [www.mangabz.com](http://www.mangabz.com)              |     ✓     |      ✓      |             Chinese              |
| Manganelo        | [manganelo.com](https://manganelo.com)                 |     ✓     |      ✓      |             English              |
| Mangareader      | [www.mangareader.net](http://www.mangareader.net)      |     ✓     |      ✓      |             English              |
| 漫画 DB          | [www.manhuadb.com](https://www.manhuadb.com)           |     ✓     |      ✓      |             Chinese              |
| 漫画堆           | [www.manhuadui.com](https://www.manhuadui.com)         |     ✓     |      ✓      |             Chinese              |
| 漫画柜           | [www.manhuagui.com](https://www.manhuagui.com)         |     ✓     |      ✓      |             Chinese              |
| 漫画铺           | [www.manhuapu.com](http://www.manhuapu.com)            |     ✓     |      ✓      |             Chinese              |
| 漫客栈           | [www.mkzhan.com](https://www.mkzhan.com)               |     ✓     |      ✓      |             Chinese              |
| nhentai          | nhentai<i>.</i>net                                     |     ✓     |      ✓      | English, Japanese, Chinese, NSFW |
| 9hentai          | 9hentai<i>.</i>com                                     |     ✓     |      ✓      |          English, NSFW           |
| 90 漫画网        | [www.90mh.com](http://www.90mh.com)                    |     ✓     |      ✓      |             Chinese              |
| 177 漫畫         | www<i>.</i>177pic<i>.</i>info                          |     ✓     |      ✓      |     Chinese, Japanese, NSFW      |
| ONE 漫画         | [www.onemanhua.com](https://www.onemanhua.com)         |     ✓     |      ✓      |             Chinese              |
| 扑飞漫画         | [www.pufei8.com](http://www.pufei8.com)                |     ✓     |      ✓      |             Chinese              |
| 奇妙漫画         | [www.qimiaomh.com](https://www.qimiaomh.com)           |     ✓     |      ✓      |             Chinese              |
| 土豪漫画         | [www.tohomh123.com](https://www.tohomh123.com)         |     ✓     |      ✓      |             Chinese              |
| TVBS 漫畫        | [www.tvbsmh.com](https://www.tvbsmh.com)               |     ✓     |      ✓      |             Chinese              |
| 台灣成人 H 漫    | twhentai<i>.</i>com                                    |     ✓     |      ✓      |     Chinese, Japanese, NSFW      |
| 二次元動漫       | [www.2animx.com](http://www.2animx.com)                |     ✓     |      ✓      |             Chinese              |
| 紳士漫畫         | www<i>.</i>wnacg<i>.</i>org                            |     ✓     |      ✓      |          Chinese, NSFW           |
| 57 漫画网        | [www.wuqimh.com](http://www.wuqimh.com/)               |     ✓     |      ✓      |             Chinese              |
| 新新漫画网       | [www.177mh.net](https://www.177mh.net)                 |     ✓     |      ✓      |             Chinese              |

Not supporting the platform you want to see? Let me know in [Issues](https://github.com/Hentioe/mikack/issues).

Sources no longer supported:

| NAME      | DOMAIN                         | REASON? |     TAGS      |
| :-------- | :----------------------------- | :-----: | :-----------: |
| 壁咚漫画  | www<i>.</i>bidongmh<i>.</i>com |  Died   | Chinese, NSFW |
| YYLS 漫畫 | [8comic.se](https://8comic.se) |  Died   |    Chinese    |
