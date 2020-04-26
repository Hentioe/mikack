use super::*;

def_regex2![
    CODE    => r#"<script>(;var siteName = "[\s\S]?";.+;)</script>"#
];

// 对 www.90mh.com 内容的抓取实现
def_extractor! {
    status	=> [
        usable: true, pageable: true, searchable: true, https: false, pageable_search: true,
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

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        let code = match_content2!(&html, &*CODE_RE)?;

        let wrap_code = format!("
            {code}

            DATA = {{path: chapterPath, list: chapterImages}};
            DATA
        ", code = code);

        let data = eval_as_obj(&wrap_code)?;
        let path = data.get_as_string("path")?;

        chapter.set_title(format!("{} {}",
            document.dom_text(".title > h1 > a")?,
            document.dom_text(".title > h2")?
        ));

        let mut addresses = vec![];
        for fname in data.get_as_array("list")?.iter() {
            let address = format!("http://img.zzszs.com.cn/{path}{fname}", path = path, fname = fname.as_string()?);
            addresses.push(address);
        }

        Ok(ChapterPages::full(chapter, addresses))
    }
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
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("绝望先生 第1话", chapter1.title);
        assert_eq!(81, chapter1.pages.len());
        let comics = extr.paginated_search("绝望先生", 1).unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
