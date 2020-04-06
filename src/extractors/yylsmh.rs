use super::*;

def_regex2![
    PARAMS   => r#"javascript:openimg\(('\d+','\d+'),'8','2'\);"#
];

/// 对 8comic.se 内容的抓取实现
def_extractor! {
    status	=> [
        usable: true, pageable: true, searchable: true, https: true,
        favicon: "https://8comic.se/favicon.ico"
    ],
    tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://8comic.se/category/連載完結/漫畫連載/page/{}/", page);

        let mut comics = itemsgen2!(
            url             = &url,
            parent_dom      = r#"div[id^="post-"]"#,
            cover_dom       = ".clip > img",
            link_dom        = ".entry-title > a"
        )?;
        comics.iter_mut().for_each(|c: &mut Comic| {
            if !c.cover.starts_with("http") {
                c.cover = format!("https:{}", &c.cover);
            }
        });

        Ok(comics)
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://8comic.se/搜尋結果/?w={}", keywords);

        itemsgen2!(
            url             = &url,
            target_dom      = ".post-list-full > li a",
            link_prefix     = "https://8comic.se"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        let html = &get(&comic.url)?.text()?;

        if comic.cover.is_empty() {
            let document = parse_document(html);
            let cover = format!("https:{}",
                document.dom_attr(".rich-content tbody > tr:first-child td:first-child > img", "src")?
            );
            comic.cover = cover;
        }

        itemsgen2!(
            html            = html,
            target_dom      = ".entry-content tbody td > a",
            link_prefix     = "https:"
        )?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.title = document.dom_text(".entry-title")?;
        let params = match_content2!(&html, &*PARAMS_RE)?;
        let runtime = include_str!("../../assets/runtime/yylsmh.js");
        let page_count = document.dom_attrs("#pull > option", "value")?.len() as i32;
        let current_src = document.dom_attr("#caonima", "src")?;
        let wrap_code = wrap_code!(runtime, format!("
            Array({page_count})
                .fill()
                .map((_, p) => p === 0 ? '{current_src}' : openimg({params}, p + 1, '{current_src}'));
        ", page_count=page_count, params=params, current_src=current_src), :end);

        let mut addresses = vec![];
        for address in eval_value(&wrap_code)?.as_array()? {
            addresses.push(format!("https:{}", address.as_string()?));
        }
        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(27, comics.len());
        let comic1 = &mut Comic::from_link("生存遊戲", "https://8comic.se/15798/");
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(15, comic1.chapters.len());
        let chapter1 = &mut Chapter::from_url("https://8comic.se/1113/");
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("火影忍者 – 556 話", chapter1.title);
        assert_eq!(15, chapter1.pages.len());
        let comics = extr.search("生存遊戲").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
