use super::*;

/// 对 manganelo.com 内容的抓取实现
def_extractor! {
	state	=> [usable: true, pageable: true, searchable: true],
	tags	=> [English],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://manganelo.com/genre-all/{}", page);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".content-genres-item",
            cover_dom       = "img.img-loading",
            link_dom        = "a.genres-item-name"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://manganelo.com/search/{}", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".search-story-item",
            cover_dom       = "img.img-loading",
            link_dom        = "a.item-title"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = ".row-content-chapter > li > a.chapter-name"
        )?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.title = document.dom_text(".panel-chapter-info-top > h1")?;
        let addresses = document.dom_attrs(".container-chapter-reader > img", "src")?;
        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(24, comics.len());

        let mut comic1 = Comic::new(
            "Naruto",
            "https://manganelo.com/manga/read_naruto_manga_online_free3",
        );
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(748, comic1.chapters.len());

        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!(
            "NARUTO VOL.1 CHAPTER 0 : NARUTO PILOT MANGA",
            chapter1.title
        );
        assert_eq!(46, chapter1.pages.len());
        let comics = extr.search("Naruto").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
