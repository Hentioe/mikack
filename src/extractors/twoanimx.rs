use super::*;

def_regex2![
    URL  => r#"(.+-cid-\d+-id-\d+)"#
];

/// 对 www.2animx.com 内容的抓取实现
def_extractor! {
    status	=> [
        usable: true, pageable: false, searchable: true, https: true,
        favicon: "https://www.2animx.com/favicon.ico"
    ],
    tags	=> [Chinese],

    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        itemsgen2!(
            url             = "https://www.2animx.com/index-update",
            parent_dom      = ".latest-list > .liemh > li",
            cover_dom       = "a > img",
            cover_prefix    = "https://www.2animx.com/",
            link_dom        = "a",
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.2animx.com/search-index?searchType=1&q={}", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".liemh > li",
            cover_dom       = "a > img",
            cover_prefix    = "https://www.2animx.com/",
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
        chapter.set_title(format!("{} - {}",
            document.dom_text(".b > a:last-child")?, document.dom_attr("img#ComicPic", "alt")?
        ));
        let prue_url = match_content2!(&chapter.url, &*URL_RE)?.to_string();
        let total = if let Ok(last_page) = document.dom_text(".lookpage > a:last-child") {
            last_page.parse::<i32>()?
        } else {
            1
        };
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
        let mut comic1 = Comic::new(
            "風雲全集",
            "https://www.2animx.com/index-comic-name-風雲全集-id-7212",
        );
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(670, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[23];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("風雲全集 - 風雲全集 第23卷", chapter1.title);
        assert_eq!(26, chapter1.pages.len());
        let chapter2 = &mut Chapter::from_url(
            "https://www.2animx.com/index-look-name-7天后發現變不回男人的幼女-cid-45376-id-574522",
        );
        extr.fetch_pages_unsafe(chapter2).unwrap();
        assert_eq!("7天后發現變不回男人的幼女 - 第1話（1P）", chapter2.title);
        assert_eq!(1, chapter2.pages.len());
        let comics = extr.search("风云全集").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
