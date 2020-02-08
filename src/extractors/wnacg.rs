use super::*;

def_regex2![
    PATH    => r#"t3\.wnacg\.download/data/t/(\d+/\d+)/\d+\.jpg"#,
    ID      => r#"https?://www\.wnacg\.org/photos-index-(page-\d+-)?aid-(\d+)\.html"#,
    FORMAT  => r#"\.([^.]+)$"#
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
        comic.chapters.push(Chapter::from_link(&comic.title, &comic.url));

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let id = match_content2!(&chapter.url, &*ID_RE, group = 2)?;
        let make_page_url = |page: usize| -> String {
            format!("https://www.wnacg.org/photos-index-page-{page}-aid-{id}.html", page = page, id = id)
        };
        chapter.url = make_page_url(1);

        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);

        chapter.set_title(document.dom_attr(".uwthumb > img", "alt")?);
        let page_count = document.dom_count(".paginator > a")? + 1;
        let path = match_content2!(
            &document.dom_attr(".uwthumb > img", "src")?,
            &*PATH_RE
        )?;

        let fetch_files = |page_document: &Html| -> Result<Vec<String>> {
            let mut files = vec![];
            for elem in page_document.select(&parse_selector(".cc > li")?) {
                let name = elem
                    .select(&parse_selector(".title > span")?)
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
                let format = match_content2!(&prevew_address, &*FORMAT_RE)?;

                files.push(format!("{}.{}", name, format));
            }
            Ok(files)
        };
        let mut files = vec![];
        let mut first_page_files = fetch_files(&document)?;
        files.append(&mut first_page_files[1..].to_vec());

        for i in 2..(page_count + 1) {
            let page_html = get(&make_page_url(i))?.text()?;
            let page_document = parse_document(&page_html);
            files.append(&mut fetch_files(&page_document)?);
        }
        let mut addresses = vec![];
        for file in files {
            addresses.push(format!(
                "https://img2.wnacg.download/data/{path}/{file}",
                path = path, file = file
            ));
        }

        Ok(ChapterPages::full(chapter, addresses))
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
        assert_eq!(27, chapter1.pages.len());
        let comics = extr.search("お姉さんが養ってあげる").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
