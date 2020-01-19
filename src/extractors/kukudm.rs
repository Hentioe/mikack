use super::*;

/// 对 comic.kukudm.com 内容的抓取实现
/// 优化空间：
/// - 复用 fetch_pages 方法的第一个 URL 内容
/// - 利用已存在当前页的下一张图片，提高 1/2 的速度
def_regex![
    TITLE_RE    => "共(\\d+)页",
    URL_RE      => "https://comic.kukudm.com/comiclist/.+(\\d+\\.htm.*)",
    IMGS_RE     => r#"\("<.+'"\+.+\+"([^']+)'>.+<.+='"\+.+\+"([^']+)'.+\);"#
];

def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("http://comic.kukudm.com/comictype/3_{}.htm", page);

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :href_prefix    => &"https://comic.kukudm.com",
            :target         => &r#"#comicmain > dd > a:nth-child(2)"#,
            :encoding       => &GBK
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"https://comic.kukudm.com",
            :target         => &"#comiclistn > dd > a:nth-child(1)",
            :encoding       => &GBK
        ]?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.decode_text(GBK)?;
        let document = parse_document(&html);
        chapter.title = document.dom_text("title")?;

        let page_counut = match_content![
            :text   => &html,
            :regex  => &*TITLE_RE
        ].parse::<usize>()?;

        let page_path = match_content![
            :text   => &chapter.url,
            :regex  => &*URL_RE
        ];
        let pure_url = chapter.url.replace(page_path, "");
        let fetch = Box::new(move |current_page| {
            let page_url = format!("{}{}.htm", pure_url, current_page);
            let page_html = get(&page_url)?.decode_text(GBK)?;
            let img_path = match_content![
                :text   => &page_html,
                :regex  => &*IMGS_RE
            ];
            let address = format!("https://s2.kukudm.com/{}", img_path);
            Ok(vec![Page::new(current_page - 1, address)])
        });

        Ok(ChapterPages::new(chapter, page_counut as i32, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(21, comics.len());

    let mut comic = Comic::new("进击的巨人", "https://comic.kukudm.com/comiclist/941/");
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(141, comic.chapters.len());

    let chapter1 = &mut comic.chapters[15];
    extr.fetch_pages_unsafe(chapter1).unwrap();
    assert_eq!("进击的巨人 番外篇2", chapter1.title);
    assert_eq!(18, chapter1.pages.len());
}
