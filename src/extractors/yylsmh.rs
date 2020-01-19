use super::*;

/// 对 8comic.se 内容的抓取实现
def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!(
            "http://8comic.se/category/%e9%80%a3%e8%bc%89%e5%ae%8c%e7%b5%90/%e6%bc%ab%e7%95%ab%e9%80%a3%e8%bc%89/page/{}/",
            page
        );

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :target         => &r#".loop-content a.clip-link"#,
            :attr_text      => &"title"
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

    // fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
    // }
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
        // let chapter1 =
        //     &mut Chapter::from_link("", "http://8comic.se/1113/");
        // extr.fetch_pages_unsafe(chapter1).unwrap();
        // assert_eq!("火影忍者 – 556 話", chapter1.title);
        // assert_eq!(15, chapter1.pages.len());
    }
}
