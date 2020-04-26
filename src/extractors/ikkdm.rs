use super::*;
use url::form_urlencoded::byte_serialize;

def_regex2![
    INDEX_NAME  => r#"(.+)\[\d+\]?$"#,
    SEARCH_NAME => r#"(.+)漫画在线$"#,
    TITLE       => "共(\\d+)页",
    URL         => r#"(https?://comic\.(ikkdm|kkkkdm)\.com/comiclist/\d+/\d+)/\d+\.htm"#,
    IMG         => r#"(?i)src='"\+.+\+"([^']+)'"#
];

// 对 comic.kkkkdm.com 内容的抓取实现
// 未来计划：
// - 可选择联通/电信
def_extractor! {
    status	=> [
        usable: true, pageable: false, searchable: true, http: true, pageable_search: true,
        favicon: "http://comic.kkkkdm.com/favicon.ico"
    ],
    tags	=> [Chinese],

    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        let url = "http://comic.kkkkdm.com/top100.htm";

        let mut comics = itemsgen2!(
            url             = url,
            parent_dom      = "#comicmain > dd",
            cover_dom       = "a > img",
            link_dom        = "a:nth-child(2)",
            link_prefix     = "http://comic.kkkkdm.com",
            encoding        = GBK
        )?;
        comics.iter_mut().for_each(|c: &mut Comic| {
            if let Ok(title) = match_content2!(&c.title, &*INDEX_NAME_RE) {
                c.title = title
            }
        });

        Ok(comics)
    }

    fn paginated_search(&self, keywords: &str, page: u32) -> Result<Vec<Comic>> {
        let keywords_bytes = &encode_text(keywords, GBK)?[..];
        let keywords_encoded: String = byte_serialize(keywords_bytes).collect();
        let url = format!("http://so.kukudm.com/search.asp?kw={}&page={}", keywords_encoded, page);

        let mut comics = itemsgen2!(
            url             = &url,
            parent_dom      = "#comicmain > dd",
            cover_dom       = "a > img",
            link_dom        = "a:nth-child(2)",
            encoding        = GBK
        )?;
        comics.iter_mut().for_each(|c: &mut Comic| {
            c.url = c.url.replace("kukudm.com", "comic.kkkkdm.com");
            if let Ok(title) = match_content2!(&c.title, &*SEARCH_NAME_RE) {
                c.title = title
            }
        });

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = "#comiclistn > dd > a:nth-child(1)",
            link_prefix     = "http://comic.kkkkdm.com",
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
            let address = format!("http://v2.kukudm.com/{}", img_path);

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
        let comic1 = &mut Comic::new("妖精的尾巴", "http://comic.kkkkdm.com/comiclist/346/");
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(654, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[3];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("妖精的尾巴 4话", chapter1.title);
        assert_eq!(20, chapter1.pages.len());
        let chapter2 = &mut Chapter::from_url("http://comic.kkkkdm.com/comiclist/2843/75129/1.htm");
        extr.fetch_pages_unsafe(chapter2).unwrap();
        assert_eq!("不死勇者罗曼史 5话", chapter2.title);
        assert_eq!(20, chapter2.pages.len());
        let comics = extr.paginated_search("妖精的尾巴", 1).unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
