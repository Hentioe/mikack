use super::*;

/// 对 www.90mh.com 内容的抓取实现
def_extractor! {
    status	=> [
        usable: true, pageable: false, searchable: true, https: false, pageable_search: true,
        favicon: "http://www.90mh.com/favicon.ico"
    ],
    tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = &format!("http://www.90mh.com/update/{}/", page);

        itemsgen2!(
            url             = url,
            parent_dom      = ".book-list > li",
            cover_dom       = ".cover > img",
            link_dom        = "a.cover",
            link_text_attr  = "title"
        )
    }

    fn paginated_search(&self, keywords: &str, page: u32) -> Result<Vec<Comic>> {
        let url = &format!("http://www.90mh.com/search/?keywords={}&page={}", keywords, page);

        itemsgen2!(
            url             = url,
            parent_dom      = ".book-list > li",
            cover_dom       = ".cover > img",
            link_dom        = "a.cover",
            link_text_attr  = "title"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        let html = &get(&comic.url)?.text()?;
        let document = parse_document(&html);

        for (i, elem) in document.select(&parse_selector(".comic-chapters")?).enumerate() { //展平所有分组
            let selector =  GroupedItemsSelector {
                document: Rc::new(parse_document(&elem.html())),
                group_dom: r#"ul[id^="chapter-list-"]"#,
                items_dom: "li > a",
                items_title_dom: "span",
                items_url_prefix: "http://www.90mh.com",
                ..Default::default()
            };
            comic.chapters.append(&mut selector.gen()?.flatten(i));
        }

        Ok(())
    }

    // fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
    // }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(comics.len(), 36);
        let comic1 = &mut Comic::new("绝望先生", "http://www.90mh.com/manhua/juewangxiansheng/");
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(74, comic1.chapters.len());
        // let chapter1 = &mut comic1.chapters[0];
        // extr.fetch_pages_unsafe(chapter1).unwrap();
        // assert_eq!("绝望先生 第1话", chapter1.title);
        // assert_eq!(81, chapter1.pages.len());
        let comics = extr.paginated_search("绝望先生", 1).unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
