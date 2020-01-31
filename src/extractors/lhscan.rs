use super::*;

/// 对 lhscan.net 内容的抓取实现
def_extractor! {[usable: true, searchable: false],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!(
            "https://lhscan.net/manga-list.html?listType=pagination&page={}&sort=last_update&sort_type=DESC",
            page
        );

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :href_prefix    => &"https://lhscan.net/",
            :target         => &r#".media > .media-body > h3 > a"#
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"https://lhscan.net/",
            :target         => &r#"td > a.chapter"#
        ]?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.title = document.dom_attr(r#"li[itemprop="itemListElement"]:last-child > a"#, "title")?;
        let addresses = document.dom_attrs(".chapter-img", "src")?;
        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(20, comics.len());
        let comic = &mut Comic::from_link(
            "Minagoroshi no Arthur - Raw",
            "https://lhscan.net/manga-ichinichi-gaishutsuroku-hanchou-raw.html",
        );
        extr.fetch_chapters(comic).unwrap();
        assert_eq!(51, comic.chapters.len());
        let chapter1 = &mut Chapter::from_link(
            "",
            "https://lhscan.net/read-ichinichi-gaishutsuroku-hanchou-raw-chapter-64.html",
        );
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!(
            "Ichinichi Gaishutsuroku Hanchou - Raw chap 64",
            chapter1.title
        );
        assert_eq!(19, chapter1.pages.len());
    }
}
