use super::*;

def_regex2![
    COVER       => r#"background-image:url\('([^']+)'\)"#,
    CHAPTER_URL => r#"(https?://www\.mangareader\.net/[^/]+/\d+)"#
];

/// 对 www.mangareader.net 内容的抓取实现
def_extractor! {
    status	=> [
        usable: true, pageable: true, searchable: true, https: false, pageable_search: true,
        favicon: "http://www.mangareader.net/favicon.ico"
    ],
    tags	=> [English],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = &format!("http://www.mangareader.net/popular/{}", (page - 1) * 30);

        let mut comics = itemsgen2!(
            url             = url,
            parent_dom      = "#mangaresults > .mangaresultitem",
            cover_dom       = ".imgsearchresults",
            cover_attr      = "style",
            link_dom        = ".manga_name h3 > a",
            link_prefix     = "http://www.mangareader.net"
        )?;

        comics.iter_mut().for_each(|comic: &mut Comic| {
            if let Ok(cover) = match_content2!(&comic.cover, &*COVER_RE) {
                comic.cover = cover.replace("r0.", "l0.").clone()
            }
        });

        Ok(comics)
    }

    fn paginated_search(&self, keywords: &str, page: u32) -> Result<Vec<Comic>> {
        let url = &format!("http://www.mangareader.net/search/?w={}&p={}", keywords, (page - 1) * 30);

        let mut comics = itemsgen2!(
            url             = url,
            parent_dom      = "#mangaresults > .mangaresultitem",
            cover_dom       = ".imgsearchresults",
            cover_attr      = "style",
            link_dom        = ".manga_name h3 > a",
            link_prefix     = "http://www.mangareader.net"
        )?;

        comics.iter_mut().for_each(|comic: &mut Comic| {
            if let Ok(cover) = match_content2!(&comic.cover, &*COVER_RE) {
                comic.cover = cover.replace("r0.", "l0.").clone()
            }
        });

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = "#listing .chico_manga + a",
            link_prefix     = "http://www.mangareader.net"
        )?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        chapter.url = match_content2!(&chapter.url, &*CHAPTER_URL_RE)?;
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);

        chapter.title = document.dom_text(".c3 > h1")?;
        let total = document.dom_count("#pageMenu > option")?;

        let fetch_address = move |page_document: &Html| -> Result<String> {
            Ok(page_document.dom_attr("#imgholder> a > img", "src")?)
        };

        let first_address = fetch_address(&document)?;
        let home_url = chapter.url.clone();

        let fetch = Box::new(move |current_page: usize| {
            let page_html = get(&format!("{}/{}", home_url, current_page))?.text()?;
            let page_document = parse_document(&page_html);
            let address = fetch_address(&page_document)?;

            Ok(vec![Page::new(current_page, address)])
        });

        Ok(ChapterPages::new(chapter, total as i32, vec![first_address], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(30, comics.len());
        let mut comic1 = Comic::new("Naruto", "http://www.mangareader.net/naruto");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(700, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("Naruto 1", chapter1.title);
        assert_eq!(53, chapter1.pages.len());
        let comics = extr.paginated_search("Naruto", 1).unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
