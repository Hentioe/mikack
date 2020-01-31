use super::*;

/// 对 manganelo.com 内容的抓取实现
def_extractor! {[usable: true, searchable: false],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = urlgen![
            :first  => &"https://manganelo.com/genre-all?type=topview",
            :next   => &"https://manganelo.com/genre-all/{}?type=topview",
            :page   => &page
        ];

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :target         => &r#".content-genres-item h3 > a.genres-item-name"#
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :target         => &".row-content-chapter > li > a.chapter-name"
        ]?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.title = document.dom_text(".panel-chapter-info-top > h1")?;
        let addresses = document.dom_attrs(".container-chapter-reader > img", "src")?;
        Ok(ChapterPages::full(chapter,  addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(24, comics.len());

    let mut comic = Comic::new("Goblin Slayer", "https://manganelo.com/manga/hgj2047065412");
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(45, comic.chapters.len());

    let chapter1 = &mut Chapter::new(
        "",
        "https://manganelo.com/chapter/hgj2047065412/chapter_1",
        0,
    );
    extr.fetch_pages(chapter1).unwrap();
    assert_eq!("GOBLIN SLAYER CHAPTER 1", chapter1.title);
    assert_eq!(50, chapter1.pages.len());
}
