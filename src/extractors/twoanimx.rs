use super::*;
use std::str;

def_regex![
    URL_RE  => r#"(.+-cid-\d+-id-\d+)"#
];

/// 对 www.2animx.com 内容的抓取实现
def_extractor! {[usable: true, pageable: false, searchable: false],
    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        itemsgen2!(
            url             = "http://www.2animx.com/index-update-date-30",
            parent_dom      = ".latest-list > .liemh > li",
            cover_dom       = "a > img",
            cover_prefix    = "http://www.2animx.com/",
            link_dom        = "a",
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = "#oneCon1 li > a"
        )?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(document.dom_attr("img#ComicPic", "alt")?);
        let prue_url = match_content![
            :text   => &chapter.url,
            :regex  => &*URL_RE
        ].to_string();
        let total = document.dom_text(".lookpage > a:last-child")?.parse::<i32>()?;
        let fetch = Box::new(move |current_page: usize| {
            let page_html = get(&format!("{}-p-{}", prue_url, current_page))?.text()?;
            let page_document = parse_document(&page_html);
            let current_addr = page_document
                .dom_attr("img#ComicPic", "src")?;
            let mut pages = vec![Page::new(current_page - 1, current_addr)];
            if current_page < total as usize {
                let next_addr = page_document
                .dom_attr(r#"#img_ad_img > img[style="display:none;"]"#, "src")?;
                pages.push(Page::new(current_page, next_addr));
            }
            Ok(pages)
        });

        Ok(ChapterPages::new(chapter, total, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert!(1 < comics.len());
        let mut comic = Comic::new(
            "風雲全集",
            "http://www.2animx.com/index-comic-name-%E9%A2%A8%E9%9B%B2%E5%85%A8%E9%9B%86-id-7212",
        );
        extr.fetch_chapters(&mut comic).unwrap();
        assert_eq!(670, comic.chapters.len());
        let chapter1 = &mut comic.chapters[23];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("風雲全集 第23卷", chapter1.title);
        assert_eq!(26, chapter1.pages.len());
    }
}
