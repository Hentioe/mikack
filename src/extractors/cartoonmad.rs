use super::*;

/// 对 www.cartoonmad.com 内容的抓取实现
/// 优化空间：
/// - 复用 fetch_pages 方法的第一个 URL 内容
/// - 过滤 fetch_pages 方法的 URL（因为被 CSS 选择器依赖，需要纯净地址）
def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = if page > 9 {
            format!("https://www.cartoonmad.com/endcm.0{}.html", page)
        }else{
            format!("https://www.cartoonmad.com/endcm.{}.html", page)
        };

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :href_prefix    => &"http://www.cartoonmad.com/",
            :target         => &r#"table[width="890"] td[colspan="2"] td[align="center"] > a"#,
            :encoding       => &BIG5
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"http://www.cartoonmad.com",
            :target         => &"fieldset td > a",
            :encoding       => &BIG5
        ]?.attach_to(comic);

        Ok(())
    }

    fn fetch_pages(&self, chapter: &mut Chapter) -> Result<()> {
        let html = get(&chapter.url)?.decode_text(BIG5)?;
        let document = parse_document(&html);
        let page_url_list = document
            .dom_attrs(r#"select[name="jump"] > option[value]"#, "value")?
            .iter()
            .map(|path| format!("http://www.cartoonmad.com/comic/{}", path))
            .collect::<Vec<_>>();
        if chapter.title.is_empty(){
            let name = document.dom_text(r#"td[width="600"] li > a:first-child"#)?;
            let chapter_text = document.dom_text(format!("a[href=\"{}\"]", &chapter.url).as_str())?;
            chapter.title = format!("{} - {}", name.replace("漫畫", ""), chapter_text);
        }

        for (i, page_url) in page_url_list.iter().enumerate() {
            let html = get(page_url)?.text()?;
            let page_document = parse_document(&html);
            let url = page_document.dom_attr(r#"a > img[oncontextmenu="return false"]"#, "src")?;
            chapter.push_page(Page::new(i, url));
        }

        Ok(())
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let mut comics = extr.index(1).unwrap();
    assert_eq!(60, comics.len());

    let mut comic = &mut comics[0];
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(411, comic.chapters.len());

    let chapter1 = &mut comic.chapters[18];
    chapter1.title = "".to_string();
    extr.fetch_pages(chapter1).unwrap();
    assert_eq!("魔導少年 - 153 話", chapter1.title);
    assert_eq!(18, chapter1.pages.len());
}
