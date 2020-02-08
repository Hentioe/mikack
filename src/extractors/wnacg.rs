use super::*;

def_regex2![
    COVER => r#"background: url("([^"]+)")"#
];

/// 对 www.wnacg.org 内容的抓取实现
def_extractor! {[usable: true, pageable: true, searchable: true],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.wnacg.org/albums-index-page-{}.html", page);

        let mut comics = itemsgen2!(
            url             = &url,
            parent_dom      = ".cc > .gallary_item",
            cover_dom       = "a > img",
            link_dom        = ".pic_box > a",
            link_prefix     = "https://www.wnacg.org",
            link_text_attr  = "title"
        )?;
        comics.iter_mut().for_each(|comic: &mut Comic| {
            comic.cover = comic.cover.replace("//", "https://");
        });

        Ok(comics)
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.wnacg.org/albums-index-page-1-sname-{}.html", keywords);

        let mut comics = itemsgen2!(
            url             = &url,
            parent_dom      = ".cc > .gallary_item",
            cover_dom       = "a > img",
            link_dom        = ".pic_box > a",
            link_prefix     = "https://www.wnacg.org",
            link_text_attr  = "title"
        )?;
        comics.iter_mut().for_each(|comic: &mut Comic| {
            comic.cover = comic.cover.replace("//", "https://");
        });

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.chapters.push(Chapter::from_link(&comic.title, &comic.url));

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
        assert_eq!(12, comics.len());
        let mut comic1 = Comic::new(
            "(C97) [てまりきゃっと (爺わら)] お姉さんが養ってあげる [绅士仓库汉化]",
            "https://www.wnacg.org/photos-index-aid-94345.html",
        );
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(1, comic1.chapters.len());
        // let chapter1 = &mut comic1.chapters[0];
        // extr.fetch_pages_unsafe(chapter1).unwrap();
        // assert_eq!(
        //     "(C97) [てまりきゃっと (爺わら)] お姉さんが養ってあげる [绅士仓库汉化]",
        //     chapter1.title
        // );
        // assert_eq!(21, chapter1.pages.len());
        let comics = extr.search("お姉さんが養ってあげる").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
