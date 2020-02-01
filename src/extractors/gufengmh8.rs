use super::*;

def_regex2![
    SCRIPT  => r#"<script>;var siteName = "";(.+);</script>"#,
];

/// 对 www.gufengmh8.com 内容的抓取实现
def_extractor! {[usable: true, searchable: true, pageable: true],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.gufengmh8.com/update/{}/", page);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".book-list > li[data-key]",
            cover_dom       = "a.cover > img",
            link_dom        = "a.cover",
            link_text_attr  = "title"
        )
    }

    fn search(&self, keyworkds: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.gufengmh8.com/search/?keywords={}", keyworkds);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".book-list > li[data-key]",
            cover_dom       = "a.cover > img",
            link_dom        = "a.cover",
            link_text_attr  = "title"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = r#"ul[id^="chapter-list-"] > li > a"#,
            target_text_dom = "span",
            link_prefix     = "https://www.gufengmh8.com"
        )?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(
            format!("{} {}", document.dom_text(".title > h1")?, document.dom_text(".title > h2")?)
        );
        let script = match_content2!(&html, &*SCRIPT_RE)?;
        let wrap_code = format!("{}
            chapterImages.map(file => `https://res.gufengmh8.com/${{chapterPath}}${{file}}`)
        ", script);
        let value = eval_value(&wrap_code)?;
        let mut addresses = vec![];
        for addr in value.as_array()? {
            addresses.push(addr.as_string()?.clone());
        }

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(36, comics.len());
        let comic1 = &mut Comic::from_link(
            "东京食尸鬼re",
            "https://www.gufengmh8.com/manhua/dongjingshishiguire/",
        );
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(137, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("东京食尸鬼re 01话", chapter1.title);
        assert_eq!(42, chapter1.pages.len());

        let comics = extr.search("东京食尸鬼").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
