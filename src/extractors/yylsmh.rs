use super::*;

def_regex![
    PARAMS_RE   => r#"javascript:openimg\(('\d+','\d+'),'8','2'\);"#
];

/// 对 8comic.se 内容的抓取实现
def_extractor! {[usable: true, searchable: false],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!(
            "http://8comic.se/category/%e9%80%a3%e8%bc%89%e5%ae%8c%e7%b5%90/%e6%bc%ab%e7%95%ab%e9%80%a3%e8%bc%89/page/{}/",
            page
        );

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :target         => &r#".loop-content a.clip-link"#,
            :text_attr      => &"title"
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :target         => &r#".entry-content tbody td > a"#
        ]?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.title = document.dom_text(".entry-title")?;
        let params = match_content![
            :text   => &html,
            :regex  => &*PARAMS_RE
        ];
        let runtime = include_str!("../../runtime/yylsmh.js");
        let page_count = document.dom_attrs("#pull > option", "value")?.len() as i32;
        let current_src = document.dom_attr("#caonima", "src")?;
        let wrap_code = wrap_code!(runtime, format!("
            Array({page_count})
                .fill()
                .map((_, p) => p === 0 ? '{current_src}' : openimg({params}, p + 1, '{current_src}'));
        ", page_count=page_count, params=params, current_src=current_src), :end);

        let mut addresses = vec![];
        for address in eval_value(&wrap_code)?.as_array()? {
            addresses.push(address.as_string()?.clone());
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
        let comic = &mut Comic::from_link("火影忍者", "http://8comic.se/1671/");
        extr.fetch_chapters(comic).unwrap();
        assert_eq!(217, comic.chapters.len());
        let chapter1 = &mut Chapter::from_link("", "http://8comic.se/1113/");
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("火影忍者 – 556 話", chapter1.title);
        assert_eq!(15, chapter1.pages.len());
    }
}
