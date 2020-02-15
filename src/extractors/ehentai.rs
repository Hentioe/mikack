use super::*;

def_regex2! {
    COUNT    => r#"Showing \d+ - \d+ of (\d+) images"#,
    URL      => r#"(https?://e-hentai\.org/g/\d+/[^/]+/)"#
}

def_extractor! {
    state	=> [usable: true, pageable: true, searchable: true],
    tags	=> [English, Japanese, Chinese, NSFW],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://e-hentai.org/?page={}", page - 1);

        itemsgen2!(
            url         = &url,
            parent_dom  = ".itg > tbody > tr:not(:nth-child(1)):not(:nth-child(15))",
            cover_dom   = ".glthumb img",
            cover_attrs = &["data-src", "src"],
            link_dom    = ".glname > a",
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://e-hentai.org/?f_search={}", keywords);

        itemsgen2!(
            url         = &url,
            parent_dom  = ".itg > tbody > tr:not(:nth-child(1)):not(:nth-child(15))",
            cover_dom   = ".glthumb img",
            cover_attrs = &["data-src", "src"],
            link_dom    = ".glname > a",
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.push_chapter(Chapter::from_link(&comic.title, &comic.url));

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);

        chapter.title = document.dom_text("#gn")?;
        let count_text = document.dom_text("div.gtb > p.gpc")?;
        let total = match_content2!(&count_text, &*COUNT_RE)?.parse::<f64>()?;
        let page_count = (total / 40.0).ceil() as u32;

        let url = match_content2!(&chapter.url, &*URL_RE)?;

        let mut view_url_list = vec![];
        for i in 0..page_count {
            let page_url = format!("{}?p={}", url, i);
            let page_html = get(&page_url)?.text()?;
            let page_docuement = parse_document(&page_html);
            let mut href_list = page_docuement.dom_attrs(".gdtm > div > a", "href")?;
            view_url_list.append(&mut href_list);
        }

        let fetch = Box::new(move |current_page| {
            let view_html = get(&view_url_list[current_page - 1])?.text()?;
            let view_docuement = parse_document(&view_html);
            let address = view_docuement.dom_attr("#img", "src")?;
            Ok(vec![Page::new(current_page - 1, address)])
        });

        Ok(ChapterPages::new(chapter, total as i32, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(25, comics.len());

        let title =
            "[Reverse Noise (Yamu)] Shizuka na Yoru ni Futarikiri (Touhou Project) [Digital]";
        let mut comic1 = Comic::from_link(title, "https://e-hentai.org/g/1550508/b913d30dcb/");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(1, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!(title, chapter1.title);
        assert_eq!(25, chapter1.pages.len());
        let comics = extr.search("Shizuka na Yoru ni Futarikiri").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
