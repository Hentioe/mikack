use super::*;
use std::cell::Cell;

def_regex![
    URL_RE      => r#"(https?://www\.177pic\.info/html/\d+/\d+/\d+\.html)"#,
    COUNT_RE    => r#".+\[(\d+)P\]$"#
];

/// 对 www.177pic.info 内容的抓取实现
/// 优化空间
/// - 复用第一页的图片数据
def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("http://www.177pic.info/page/{}/", page);

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :target         => &r#".post > .picture-box > h2 > a"#
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.push_chapter(Chapter::from_link(&comic.title, &comic.url));
        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let pure_url = match_content![
            :text   => &chapter.url,
            :regex  => &*URL_RE
        ].to_string();
        let html = get(&pure_url)?.text()?;
        let document = parse_document(&html);
        chapter.title = Chapter::title(document.dom_text(".entry-title")?);

        let total = match_content![
            :text   => &chapter.title,
            :regex  => &*COUNT_RE
        ].parse::<i32>()?;

        let last_page_end = Cell::new(0);
        let next_page = Cell::new(0);
        let fetch = Box::new(move |current_page: usize|{
            if last_page_end.get() < current_page {
                next_page.set(next_page.get() + 1);
            }
            let page_html = get(&format!("{}/{}/", pure_url, next_page.get()))?.text()?;
            let page_document = parse_document(&page_html);
            let addresses = page_document.dom_attrs(".single-content > p > img", "data-lazy-src")?;
            let mut pages = vec![];
            for (i, address) in addresses.iter().enumerate() {
                pages.push(Page::new(i + current_page - 1, address));
            }
            last_page_end.set(last_page_end.get() + pages.len());
            Ok(pages)
        });

        Ok(ChapterPages::new(chapter, total, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let mut comics = extr.index(1).unwrap();
    assert_eq!(28, comics.len());

    let comic = &mut comics[0];
    extr.fetch_chapters(comic).unwrap();
    assert_eq!(1, comic.chapters.len());

    let chapter1 = &mut Chapter::from_link("", "http://www.177pic.info/html/2020/01/3254890.html");
    extr.fetch_pages_unsafe(chapter1).unwrap();
    assert_eq!("[だらぶち] 絶対強者 ch.1-4 [66P]", chapter1.title);
    assert_eq!(66, chapter1.pages.len());
}
