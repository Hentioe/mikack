use super::*;
use reqwest::header::CONTENT_TYPE;
use url::form_urlencoded::byte_serialize;

/// 对 www.cartoonmad.com 内容的抓取实现
/// 优化空间：
/// - 复用 pages_iter 方法的第一个 URL 内容
def_extractor! {[usable: true, pageable: true, searchable: true],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = if page > 9 {
            format!("https://www.cartoonmad.com/newcm.0{}.html", page)
        }else{
            format!("https://www.cartoonmad.com/newcm.{}.html", page)
        };

        itemsgen2!(
            url             = &url,
            parent_dom      = r#"table[width="890"] td[colspan="2"] td[align="center"]"#,
            cover_dom       = "img",
            cover_prefix    = "https://www.cartoonmad.com",
            link_dom        = "a.a1",
            link_prefix     = "https://www.cartoonmad.com/",
            encoding        = BIG5
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = "https://www.cartoonmad.com/search.html";
        let keyword_bytes = &encode_text(keywords, BIG5)?[..];
        let keyword_encoded: String = byte_serialize(keyword_bytes).collect();

        let client = Client::new();
        let html = client
            .post(url)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(format!("keyword={}&searchtype=all", keyword_encoded))
            .send()?
            .decode_text(BIG5)?;

        itemsgen2!(
            html            = &html,
            parent_dom      = r#"table[width="890"] td[colspan="2"] td[align="center"]"#,
            cover_dom       = "img",
            cover_prefix    = "https://www.cartoonmad.com",
            link_dom        = "a.a1",
            link_prefix     = "https://www.cartoonmad.com/"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = &"fieldset td > a",
            link_prefix     = &"http://www.cartoonmad.com",
            encoding        = &BIG5
        )?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.decode_text(BIG5)?;
        let mut pure_url = chapter.url.clone().replace("https", "http");
        if let Some(query_params_index) = pure_url.find("?") {
            pure_url = pure_url[0..query_params_index].to_string();
        }
        let document = parse_document(&html);
        let page_url_list: Vec<String> = document
            .dom_attrs(r#"select[name="jump"] > option[value]"#, "value")?
            .iter()
            .map(|path| format!("http://www.cartoonmad.com/comic/{}", path))
            .collect::<Vec<String>>();
        let name = document.dom_text(r#"td[width="600"] li > a:first-child"#)?;
        let chapter_text = document.dom_text(format!("a[href=\"{}\"]", &pure_url).as_str())?;
        chapter.title = format!("{} - {}", name.replace("漫畫", ""), chapter_text);
        let len = page_url_list.len() as i32;
        let fetch = Box::new(move |current_page| {
            let html = get(&page_url_list[current_page - 1])?.text()?;
            let page_document = parse_document(&html);
            let src = page_document.dom_attr(r#"a > img[oncontextmenu="return false"]"#, "src")?;
            let address = format!("https://www.cartoonmad.com/comic/{}", src);
            Ok(vec![Page::new(current_page - 1, address)])
        });

        Ok(ChapterPages::new(chapter, len, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(60, comics.len());
        let comic1 =
            &mut Comic::from_link("魔導少年", "https://www.cartoonmad.com/comic/1153.html");
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(411, comic1.chapters.len());
        let chapter1 =
            &mut Chapter::from_link("", "https://www.cartoonmad.com/comic/115301532018001.html");
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("魔導少年 - 153 話", chapter1.title);
        assert_eq!(18, chapter1.pages.len());
        let comics = extr.search("魔導少年").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
