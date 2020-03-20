use super::*;

/// 对 www.wuqimh.com 内容的抓取实现
def_extractor! {
    status	=> [
        usable: false, pageable: false, searchable: true, https: false, paginable_search: true,
        favicon: "http://www.wuqimh.com/favicon.ico"
    ],
    tags	=> [Chinese, NSFW],

    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        let url = "http://www.wuqimh.com/latest/";

        itemsgen2!(
            url             = url,
            parent_dom      = ".latest-list > ul > li",
            cover_dom       = "a > img",
            cover_attr      = "data-src",
            link_dom        = ".ell > a",
            link_prefix     = "http://www.wuqimh.com"
        )
    }

    fn paginated_search(&self, keywords: &str, page: u32) -> Result<Vec<Comic>> {
        let url = format!("http://www.wuqimh.com/search/q_{}-p-{}", keywords, page);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".book-result > li",
            cover_dom       = ".book-cover > .bcover > img",
            link_dom        = ".book-detail dt > a",
            link_prefix     = "http://www.wuqimh.com"
        )
    }

    // fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
    // }

    // fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
    // }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(457, comics.len());
        let comic1 = &mut Comic::new("进击的巨人", "http://www.wuqimh.com/118/");
        // extr.fetch_chapters(&comic1).unwrap();
        // assert_eq!(0, comic1.chapters.len());
        // let chapter1 = &mut comic1.chapters[0];
        // extr.fetch_pages_unsafe(chapter1).unwrap();
        // assert_eq!("进击的巨人00话", chapter1.title);
        // assert_eq!(35, chapter1.pages.len());
        let comics = extr.paginated_search("进击的巨人", 1).unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
