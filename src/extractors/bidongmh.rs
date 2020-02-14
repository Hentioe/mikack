use super::*;

def_regex2![
    COVER => r#"background: url("([^"]+)")"#
];

/// 对 www.bidongmh.com 内容的抓取实现
def_extractor! {[usable: true, pageable: true, searchable: true],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.bidongmh.com/update?page={}", page);

        itemsgen2!(
            url             = &url,
            target_dom      = ".item > .chapter",
            link_prefix     = "https://www.bidongmh.com",
            link_text_attr  = "title"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.bidongmh.com/search?keyword={}", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".comic-list > .item",
            cover_dom       = "a > img",
            cover_attr      = "data-src",
            link_dom        = "h3.title > a",
            link_prefix     = "https://www.bidongmh.com"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = ".chapter-list > .item > a",
            link_prefix     = "https://www.bidongmh.com"
        )?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(format!("{} {}",
            document.dom_text(".crumbs > a:nth-child(2)")?,
            document.dom_text(".title")?.split("#")
                .next()
                .ok_or(err_msg("No title found"))?
        ));
        let addresses = document.dom_attrs(".comiclist > .comicpage > img", "src")?;
        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(22, comics.len());
        let mut comic1 = Comic::new("恋爱大排档", "https://www.bidongmh.com/book/256");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(16, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("恋爱大排档 第1话", chapter1.title);
        assert_eq!(21, chapter1.pages.len());
        let comics = extr.search("恋爱大排档").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
