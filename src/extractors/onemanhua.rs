use super::*;

/// 对 www.onemanhua.com 内容的抓取实现
def_extractor! {
    status	=> [
        usable: true, pageable: true, searchable: true, https: true,
        favicon: "https://www.onemanhua.com/favicon.png"
    ],
    tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.onemanhua.com/show?orderBy=update&page={}", page);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".fed-list-info > .fed-list-item",
            cover_dom       = "a.fed-list-pics",
            cover_attr      = "data-original",
            link_dom        = "a.fed-list-title",
            link_prefix     = "https://www.onemanhua.com"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.onemanhua.com/search?searchString={}", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".fed-deta-info",
            cover_dom       = "a.fed-list-pics",
            cover_attr      = "data-original",
            link_dom        = "h1 > a",
            link_prefix     = "https://www.onemanhua.com"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = ".all_data_list > ul > li > a",
            link_prefix     = "https://www.onemanhua.com"
        )?.reversed_attach_to(comic);

        Ok(())
    }

    // fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
    //     let html = get(&chapter.url)?.text()?;
    //     let document = parse_document(&html);
    // }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(30, comics.len());
        let mut comic1 = Comic::new("最后的召唤师", "https://www.onemanhua.com/12436/");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(418, comic1.chapters.len());
        // let chapter1 = &mut comic1.chapters[0];
        // extr.fetch_pages_unsafe(chapter1).unwrap();
        // assert_eq!("最后的召唤师 第1话1 契约", chapter1.title);
        // assert_eq!(0, chapter1.pages.len());
        let comics = extr.search("最后的召唤师").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
