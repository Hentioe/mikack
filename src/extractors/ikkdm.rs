use super::*;
use url::form_urlencoded::byte_serialize;

def_regex2![
    NAME        => r#"(.+)\[\d+\]?$"#,
    NAME2       => r#"(.+)漫画(电信|联通)$"#,
    TITLE       => "共(\\d+)页",
    URL         => r#"(https?://comic\.kukudm\.com/comiclist/\d+/\d+)/\d+\.htm"#,
    IMG         => r#"src='"\+server\+"([^']+)'>""#
];

/// 对 comic.ikkdm.com 内容的抓取实现
/// 未来计划：
/// - 可选择联通/电信
def_extractor! {
    status	=> [
        usable: true, pageable: false, searchable: true, http: true,
        favicon: "http://comic.ikkdm.com/favicon.ico"
    ],
    tags	=> [Chinese],

    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        let url = "http://comic.ikkdm.com/top100.htm";

        let mut comics = itemsgen2!(
            url             = url,
            parent_dom      = "#comicmain > dd",
            cover_dom       = "a > img",
            link_dom        = "a:nth-child(2)",
            link_prefix     = "http://comic.ikkdm.com",
            encoding        = GBK
        )?;
        comics.iter_mut().for_each(|c: &mut Comic| {
            if let Ok(title) = match_content2!(&c.title, &*NAME_RE) {
                c.title = title
            }
        });

        Ok(comics)
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let keywords_bytes = &encode_text(keywords, GBK)?[..];
        let keywords_encoded: String = byte_serialize(keywords_bytes).collect();
        let url = format!("http://so.kukudm.com/search.asp?kw={}", keywords_encoded);

        let mut comics = itemsgen2!(
            url             = &url,
            parent_dom      = "#comicmain > dd",
            cover_dom       = "a > img",
            link_dom        = "a:nth-child(2)",
            encoding        = GBK
        )?;
        comics.iter_mut().for_each(|c: &mut Comic| {
            if let Ok(title) = match_content2!(&c.title, &*NAME2_RE) {
                c.title = title
            }
        });

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = "#comiclistn > dd > a:nth-child(1)",
            link_prefix     = "http://comic.ikkdm.com",
            encoding        = GBK
        )?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let pure_url = match_content2!(&chapter.url, &*URL_RE)?;
        chapter.url = format!("{}/1.htm", pure_url);
        let html = get(&chapter.url)?.decode_text(GBK)?;
        let document = parse_document(&html);
        chapter.set_title(document.dom_text("title")?);

        let fetch_page = |page_html: &str| -> Result<String> {
            let img_path = match_content2!(page_html, &*IMG_RE)?;
            let address = format!("http://s2.kukudm.com/{}", img_path);

            Ok(address)
        };

        let first_address = fetch_page(&html)?;
        let page_counut = match_content2!(&html, &*TITLE_RE)?.parse::<usize>()?;

        let fetch = Box::new(move |current_page| {
            let page_url = format!("{}/{}.htm", pure_url, current_page);
            let page_html = get(&page_url)?.decode_text(GBK)?;
            let address = fetch_page(&page_html)?;
            Ok(vec![Page::new(current_page - 1, address)])
        });

        Ok(ChapterPages::new(chapter, page_counut as i32, vec![first_address], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(100, comics.len());
        let mut comic1 = Comic::new("妖精的尾巴", "https://kukudm.com/comiclist/346/");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(652, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[3];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("妖精的尾巴 4话", chapter1.title);
        assert_eq!(20, chapter1.pages.len());
        let comics = extr.search("妖精的尾巴").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
