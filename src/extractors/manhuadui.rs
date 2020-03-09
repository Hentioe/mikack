use super::*;

def_regex2! [
    ENCODE_TEXT => r#"var\s*chapterImages\s*=\s*"([^"]+)""#,
    PATH        => r#"var\s*chapterPath\s*=\s*"([^"]+)""#
];

def_extractor! {
    status	=> [
        usable: true, pageable: true, searchable: true, https: true,
        favicon: "https://www.manhuadui.com/favicon.ico"
    ],
    tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.manhuadui.com/update/{}/", page);

        itemsgen2!(
            url         = &url,
            parent_dom  = ".list_con_li > li",
            cover_dom   = ".comic_img > img",
            link_dom    = "h3 > a"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.manhuadui.com/search/?keywords={}", keywords);

        itemsgen2!(
            url         = &url,
            parent_dom  = ".list_con_li > li",
            cover_dom   = ".image-link > img",
            link_dom    = "p > a"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = "ul.list_con_li > li > a",
            link_prefix     = "https://www.manhuadui.com",
            link_text_dom   = "span.list_con_zj"
        )?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        let comic_name = document.dom_text(".head_title > h1 > a")?;
        let chapter_num = document.dom_text(".head_title > h2")?;
        chapter.title = format!("{} {}", comic_name, chapter_num);

        let encode_text = match_content2!(&html, &*ENCODE_TEXT_RE)?;

        let runtime = format!("{}\n{}", include_str!("../../assets/lib/crypto-js.js"), include_str!("../../assets/runtime/manhuadui.js"));
        let wrap_code = wrap_code!(runtime, format!("
            var chapterImages =
            \"{}\";

            decrypt20180904(chapterImages)
        ", encode_text), :end);

        let mut addresses = vec![];
        if let Ok(path) = match_content2!(&html, &*PATH_RE) {
            for fname in eval_value(&wrap_code)?.as_array()? {
                let address = format!("https://img01.eshanyao.com/{}/{}", path, fname.as_string()?);
                addresses.push(address);
            }

        } else {
            let mut headers_updated = false;
            for addr in eval_value(&wrap_code)?.as_array()? {
                let addr = addr.as_string()?.clone();
                if !headers_updated {
                    if addr.starts_with("http://images.dmzj.com") {
                        chapter.page_headers.clear();
                        chapter.page_headers.insert(
                            String::from("Referer"),
                            String::from("https://manhua.dmzj.com/")
                        );
                    }
                    headers_updated = true;
                }
                addresses.push(addr);
            }
        }

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(36, comics.len());

        let mut comic1 = Comic::from_link(
            "爱管闲事的JK与只有头的杜拉罕",
            "https://www.manhuadui.com/manhua/aiguanxianshideJKyuzhiyoutoudedulahan/",
        );
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(9, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("爱管闲事的JK与只有头的杜拉罕 01话", chapter1.title);
        assert_eq!(27, chapter1.pages.len());
        let comics = extr.search("爱管闲事的JK与只有头的杜拉罕").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
