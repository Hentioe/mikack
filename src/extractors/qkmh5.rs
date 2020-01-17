use super::*;
use base64::decode;
use std::str;

def_regex![
    ENCODE_TEXT_RE => r#"var qTcms_S_m_murl_e="([^"]+)";"#
];

/// 对 www.qkmh5.com 内容的抓取实现
def_exctractor! {
    fn is_usable(&self) -> bool { true }
    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        let url = "http://www.qkmh5.com/mall/".to_string();

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :target         => &r#"li > a.tip_img"#,
            :encoding       => &GBK
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :target         => &".plist ul > li > a",
            :encoding       => &GBK
        ]?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.decode_text(GBK)?;
        let document = parse_document(&html);
        chapter.title = document.dom_text(r#"a[name="lookpic"] + h1"#)?;
        let encode_text = match_content![
            :text   => &html,
            :regex  => &*ENCODE_TEXT_RE
        ];
        let decode_bytes = &decode(encode_text)?[..];
        let decode_text = str::from_utf8(decode_bytes)?;

        let mut addresses: Vec<String> = vec![];
        for address in decode_text.split("$qingtiandy$") {
            addresses.push(address.to_string());
        }
        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(2283, comics.len());

    let mut comic = Comic::new("爱丽丝学园", "http://www.qkmh5.com/mh/ailisixueyuan.html");
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(147, comic.chapters.len());

    let chapter1 = &mut comic.chapters[0];
    chapter1.title = "".to_string();
    extr.fetch_pages(chapter1).unwrap();
    assert_eq!("爱丽丝学园第180话", chapter1.title);
    assert_eq!(64, chapter1.pages.len());
}
