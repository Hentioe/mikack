use super::*;

/// 对 www.mkzhan.com 内容的抓取实现
def_extractor! {
    status	=> [
        usable: true, pageable: false, searchable: true, https: true, pageable_search: true,
        favicon: "https://www.mkzhan.com/favicon.ico"
    ],
    tags	=> [Chinese],

    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        let url = "https://www.mkzhan.com/update/";

        itemsgen2!(
            url             = url,
            parent_dom      = ".update-list > .common-comic-item",
            cover_dom       = ".cover > img",
            cover_attr      = "data-src",
            link_dom        = ".comic__title > a",
            link_prefix     = "https://www.mkzhan.com"
        )
    }

    fn paginated_search(&self, keywords: &str, page: u32) -> Result<Vec<Comic>> {
        let url = &format!("https://www.mkzhan.com/search/?keyword={}&page={}", keywords, page);

        itemsgen2!(
            url             = url,
            parent_dom      = ".search-comic-list > .common-comic-item",
            cover_dom       = ".cover > img",
            cover_attr      = "data-src",
            link_dom        = ".comic__title > a",
            link_prefix     = "https://www.mkzhan.com"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = ".j-chapter-item > .j-chapter-link",
            link_attr       = "data-hreflink",
            link_prefix     = "https://www.mkzhan.com",
            ignore_contains = ".vip-tag" // 排除付费章节
        )?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);

        chapter.title = format!("{} {}",
            document.dom_text("a.j-comic-title")?,
            document.dom_text("a.last-crumb")?,
        );

        let addresses = document.dom_attrs(".rd-article__pic > img", "data-src")?;

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert!(comics.len() > 0);
        let comic1 = &mut Comic::new("绝品小神医", "https://www.mkzhan.com/212800/");
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(40, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("绝品小神医 第1话", chapter1.title);
        assert_eq!(25, chapter1.pages.len());
        let comics = extr.paginated_search("绝品小神医", 1).unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
