use super::*;

def_regex! {
    COUNT_RE    => r#"Showing \d+ - \d+ of (\d+) images"#,
    URL_RE      => r#"(https?://e-hentai\.org/g/\d+/[^/]+/)"#
}

def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = urlgen![
            :first  => &"https://e-hentai.org/",
            :next   => &"https://e-hentai.org/?page={}",
            :page   => &page
        ];

        itemsgen![
            :entry      => Comic,
            :url        => &url,
            :selector   => &"tbody > tr > td.gl3c.glname"
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.push_chapter(Chapter::from_link(&comic.title, &comic.url));
        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);

        chapter.title = document.dom_text("#gn")?;
        let count_text = document.dom_text("div.gtb > p.gpc")?;
        let total = match_content![
            :text   => &count_text,
            :regex  => &*COUNT_RE
        ].parse::<f64>()?;
        let page_count = (total / 40.0).ceil() as u32;

        let url = match_content![
            :text   => &chapter.url,
            :regex  => &*URL_RE
        ];

        let mut view_url_list = vec![];
        for i in 0..page_count {
            let page_url = format!("{}?p={}", url, i);
            let page_html = get(&page_url)?.text()?;
            let page_docuement = parse_document(&page_html);
            let mut href_list = page_docuement.dom_attrs(".gdtm > div > a", "href")?;
            view_url_list.append(&mut href_list);
        }

        let fetch = Box::new(move |current_page| {
            let view_html = get(&view_url_list[current_page - 1])?.text()?;
            let view_docuement = parse_document(&view_html);
            let address = view_docuement.dom_attr("#img", "src")?;
            Ok(vec![Page::new(current_page - 1, address)])
        });

        Ok(ChapterPages::new(chapter, total as i32, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(25, comics.len());

    let title = "[Reverse Noise (Yamu)] Shizuka na Yoru ni Futarikiri (Touhou Project) [Digital]";

    let mut comic = Comic::from_link(title, "https://e-hentai.org/g/1550508/b913d30dcb/");
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(1, comic.chapters.len());

    let chapter1 = &mut comic.chapters[0];
    extr.fetch_pages_unsafe(chapter1).unwrap();
    assert_eq!(title, chapter1.title);
    assert_eq!(25, chapter1.pages.len());
}
