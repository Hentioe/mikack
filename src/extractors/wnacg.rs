use super::*;

def_regex2![
    SERVER  => r#"//(.+\..+\..+)/data/t/"#,
    PATH    => r#"//.+\..+\..+/data/t/(\d+/\d+)/\d+\..+"#,
    ID      => r#"https?://www\.wnacg\.org/photos-index-(page-\d+-)?aid-(\d+)\.html"#,
    FORMAT  => r#"\.([^.]+)$"#,
    COUNT   => r#"頁數：(\d+)P"#
];

/// 对 www.wnacg.org 内容的抓取实现
def_extractor! {[usable: true, pageable: true, searchable: true],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.wnacg.org/albums-index-page-{}.html", page);

        let mut comics = itemsgen2!(
            url             = &url,
            parent_dom      = ".cc > .gallary_item",
            cover_dom       = "a > img",
            link_dom        = ".pic_box > a",
            link_prefix     = "https://www.wnacg.org",
            link_text_attr  = "title"
        )?;
        comics.iter_mut().for_each(|comic: &mut Comic| {
            comic.cover = comic.cover.replace("//", "https://");
        });

        Ok(comics)
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.wnacg.org/albums-index-page-1-sname-{}.html", keywords);

        let mut comics = itemsgen2!(
            url             = &url,
            parent_dom      = ".cc > .gallary_item",
            cover_dom       = "a > img",
            link_dom        = ".pic_box > a",
            link_prefix     = "https://www.wnacg.org",
            link_text_attr  = "title"
        )?;
        comics.iter_mut().for_each(|comic: &mut Comic| {
            comic.cover = comic.cover.replace("//", "https://");
        });

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.chapters.push(Chapter::from(&*comic));

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let id = match_content2!(&chapter.url, &*ID_RE, group = 2)?;
        let make_page_url = move |page: usize| -> String {
            format!("https://www.wnacg.org/photos-index-page-{page}-aid-{id}.html", page = page, id = id)
        };
        chapter.url = make_page_url(1);

        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);

        chapter.set_title(document.dom_attr(".uwthumb > img", "alt")?);
        let path = match_content2!(
            &document.dom_attr(".uwthumb > img", "src")?,
            &*PATH_RE
        )?;
        let total = match_content2!(
            &document.dom_text(".uwconn > label:nth-child(2)")?,
            &*COUNT_RE
        )?.parse::<i32>()?;

        let fetch_page_addresses = move |page_document: &Html| -> Result<Vec<String>> {
            let mut page_addresses = vec![];
            for elem in page_document.select(&parse_selector(".cc > li")?) {
                let name = elem
                    .select(&parse_selector(".title > span:last-child")?)
                    .next()
                    .ok_or(err_msg("No name DOM found"))?
                    .text()
                    .next()
                    .ok_or(err_msg("No name text found"))?
                    .trim();
                let prevew_address = elem
                    .select(&parse_selector(".pic_box > a > img")?)
                    .next()
                    .ok_or(err_msg("No preview-image DOM found"))?
                    .value()
                    .attr("src")
                    .ok_or(err_msg("No preview-image src found"))?;
                let ext = match_content2!(&prevew_address, &*FORMAT_RE)?;
                let file = format!("{}.{}", name, ext);
                let server = match &match_content2!(&*prevew_address, &*SERVER_RE)?[..] {
                    "t1.wnacg.download" => "img1.wnacg.download",
                    _ => "img2.wnacg.download" // 已知的有 t2/t3 服务器映射到 img2
                };
                let address = format!(
                    "https://{server}/data/{path}/{file}",
                    server = server, path = path, file = file
                );

                page_addresses.push(address);
            }
            Ok(page_addresses)
        };
        let first_page_addresses = fetch_page_addresses(&document)?;
        let fetch = Box::new(move |current_page: usize| {
            let page_num = (current_page as f64 / 12.0).ceil() as usize;
            let page_html = get(&make_page_url(page_num))?.text()?;
            let page_document = parse_document(&page_html);
            let mut pages = vec![];
            for (i, addr) in fetch_page_addresses(&page_document)?.iter().enumerate() {
                pages.push(Page::new(current_page + i, addr))
            }

            Ok(pages)
        });

        Ok(ChapterPages::new(chapter, total, first_page_addresses, fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(12, comics.len());
        let mut comic1 = Comic::new(
            "(C97) [てまりきゃっと (爺わら)] お姉さんが養ってあげる [绅士仓库汉化]",
            "https://www.wnacg.org/photos-index-aid-94345.html",
        );
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(1, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!(
            "(C97) [てまりきゃっと (爺わら)] お姉さんが養ってあげる [绅士仓库汉化]",
            chapter1.title
        );
        assert_eq!(28, chapter1.pages.len());
        let comics = extr.search("お姉さんが養ってあげる").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
