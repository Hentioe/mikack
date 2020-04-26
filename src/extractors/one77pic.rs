use super::*;
use std::cell::Cell;

def_regex2![
    URL      => r#"(https?://www\.177pic\.info/html/\d+/\d+/\d+\.html)"#,
    COUNT    => r#".+\[(\d+)P\]$"#
];

// 对 www.177pic.info 内容的抓取实现
def_extractor! {
    status	=> [
        usable: true, pageable: true, searchable: true, https: false, pageable_search: true,
        favicon: "http://www.177pic.info/wp-content/themes/azzxx/img/favicon.ico"
    ],
    tags	=> [Chinese, Japanese, NSFW],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("http://www.177pic.info/page/{}/", page);

        itemsgen2!(
            url             = &url,
            parent_dom      = r#"article[id^="post-"]"#,
            cover_dom       = "img",
            link_dom        = "h2 > a"
        )
    }

    fn paginated_search(&self, keywords: &str, page: u32) -> Result<Vec<Comic>> {
        let url = format!("http://www.177pic.info/page/{}/?s={}", page, keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = r#"article.picture"#,
            cover_dom       = "a > img",
            link_dom        = "h2 > a"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.push_chapter(Chapter::from(&*comic));

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let pure_url = match_content2!(&chapter.url, &*URL_RE)?;
        chapter.url = pure_url.clone();
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(document.dom_text(".entry-title")?);

        let total = match_content2!(&chapter.title, &*COUNT_RE)?.parse::<i32>()?;

        let last_page_end = Cell::new(0);
        let next_page = Cell::new(0);
        let fetch_page = move |current_page: usize| -> Result<Vec<String>> {
            if last_page_end.get() < current_page {
                next_page.set(next_page.get() + 1);
            }
            let page_html = get(&format!("{}/{}/", pure_url, next_page.get()))?.text()?;
            let page_document = parse_document(&page_html);
            let addresses = page_document
                .dom_attrs(".single-content > p > img", "data-lazy-src")?
                .iter()
                .map(|addr| {
                    addr.clone()
                })
                .collect::<Vec<_>>();
            last_page_end.set(last_page_end.get() + addresses.len());

            Ok(addresses)
        };
        let first_addresses = fetch_page(1)?;
        let fetch = Box::new(move |current_page: usize|{
            let pages = fetch_page(current_page)?
                .iter()
                .enumerate()
                .map(|(i, addr)| {
                    Page::new(i + current_page - 1, addr.clone())
                })
                .collect::<Vec<_>>();

            Ok(pages)
        });

        Ok(ChapterPages::new(chapter, total, first_addresses, fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(28, comics.len());

        let comic1_title = "[だらぶち] 絶対強者 ch.1-4 [66P]";
        let comic1 = &mut Comic::from_link(
            comic1_title,
            "http://www.177pic.info/html/2020/01/3254890.html",
        );
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(1, comic1.chapters.len());

        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!(comic1_title, chapter1.title);
        assert_eq!(66, chapter1.pages.len());
        let comics = extr.paginated_search("絶対強者", 1).unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
