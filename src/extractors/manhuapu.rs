use super::*;
use base64;
use std::str;

def_regex2![
    DATA    => r#"qTcms_S_m_murl_e="([^"]+)""#
];

/// 对 www.manhuapu.com 内容的抓取实现
def_extractor! {
	status	=> [
		usable: true, pageable: false, searchable: true, https: false
	],
	tags	=> [Chinese],

    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        let url = "http://www.manhuapu.com/new/";

        itemsgen2!(
            url             = url,
            parent_dom      = ".updateList > ul > li",
            cover_dom       = "a.video",
            cover_attr      = "i",
            link_dom        = "a.video",
            link_prefix     = "http://www.manhuapu.com"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.manhuapu.com/statics/search.aspx?key={}", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".dmList > ul > li",
            cover_dom       = ".pic > img",
            link_dom        = "dt > a",
            link_prefix     = "http://www.manhuapu.com"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = ".plist > ul > li > a",
            link_prefix     = "http://www.manhuapu.com"
        )?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(format!("{} {}",
            document.dom_text(".title h1 > a")?,
            document.dom_text(".title h2")?
        ));
        let data = match_content2!(&html, &*DATA_RE)?;
        let data_bytes = &base64::decode(&data)?[..];
        let addresses = str::from_utf8(data_bytes)?
            .split("$qingtiandy$")
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(100, comics.len());
        let mut comic1 = Comic::new("侠行九天", "http://www.manhuapu.com/rexue/xiaxingjiutian/");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(156, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("侠行九天 番外1 身世篇", chapter1.title);
        assert_eq!(30, chapter1.pages.len());
        let comics = extr.search("侠行九天").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
