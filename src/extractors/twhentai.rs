use super::*;

def_regex2![
    PAGE_NUM    => r#"/[^/]+/\d+_p(\d+)/"#,
    URL         => r#"(https?://twhentai\.com/[^/]+/\d+)"#
];

/// 对 www.onemanhua.com 内容的抓取实现
/// 优化空间
/// - 服用最后一页的数据
def_extractor! {
    status	=> [
        usable: true, pageable: true, searchable: true, https: false
    ],
    tags	=> [Chinese, NSFW],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("http://twhentai.com/hentai_doujin/page_{}.html", page);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".recommended-grids .resent-grid",
            cover_dom       = ".thumbnail > img",
            cover_prefix    = "http://twhentai.com",
            link_dom        = "h5 > a",
            link_prefix     = "http://twhentai.com"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("http://twhentai.com/search/{}/", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".recommended-grids .resent-grid",
            cover_dom       = ".thumbnail > img",
            cover_prefix    = "http://twhentai.com",
            link_dom        = "h5 > a",
            link_prefix     = "http://twhentai.com"
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

        chapter.set_title(document.dom_text(".heading > h3")?);
        // 获取单页的所有资源地址
        let get_page_addresses = move |page_document: &Html| -> Result<Vec<String>> {
            let page_addresses = page_document.dom_attrs(".recommended-grid-img > .thumbnail > img", "src")?
                .iter()
                .map(|page| {
                    format!("http://twhentai.com{}", page.replace("-thumb265x385", "").replace("imglink", "imgwlink"))
                })
                .collect::<Vec<_>>();

            Ok(page_addresses)
        };
        // 计算总数
        let last_page_num = match_content2!(&document.dom_attr(".pagination > li:last-child > a", "href")?, &*PAGE_NUM_RE)?.parse::<usize>()?;
        let last_page_html = get(&format!("{}_p{}/", pure_url, last_page_num))?.text()?;
        let last_page_document = parse_document(&last_page_html);
        let total = get_page_addresses(&last_page_document)?.len() + (last_page_num - 1) * 16;

        let first_addresses = get_page_addresses(&document)?;

        let fetch = Box::new(move |current_page: usize| {
            let page_num = (current_page as f64 / 16.0f64).ceil() as usize;
            let page_url = format!("{}_p{}/", pure_url, page_num);
            let page_html = get(&page_url)?.text()?;
            let page_document = parse_document(&page_html);
            let pages = get_page_addresses(&page_document)?
                .iter()
                .enumerate()
                .map(|(i, addr)| Page::new((page_num - 1) * 16 + i, addr))
                .collect::<Vec<_>>();

            Ok(pages)
        });

        Ok(ChapterPages::new(chapter, total as i32, first_addresses, fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(16, comics.len());
        let comic1 = &mut Comic::new("[中文H漫](#にじそうさく3) [French letter (藤崎ひかり)] シスタークレアと秘密の催眠アプリ (シスター・クレア) [中国翻訳]", "http://twhentai.com/hentai_doujin/68098/");
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(1, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("[中文H漫](#にじそうさく3) [French letter (藤崎ひかり)] シスタークレアと秘密の催眠アプリ (シスター・クレア) [中国翻訳]", chapter1.title);
        assert_eq!(23, chapter1.pages.len());
        let comics = extr.search("秘密の催眠").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
