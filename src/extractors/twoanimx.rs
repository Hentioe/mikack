use super::*;
use std::str;

/// 对 www.2animx.com 内容的抓取实现
def_exctractor! {
    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        itemsgen2!(
            url             = "http://www.2animx.com/index-update-date-30",
            parent_dom      = ".latest-list > .liemh > li",
            cover_dom       = "a > img",
            cover_prefix    = "http://www.2animx.com/",
            link_dom        = "a",
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = "#oneCon1 li > a"
        )?.attach_to(comic);

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
        assert!(1 < comics.len());
        let mut comic = Comic::new(
            "風雲全集",
            "http://www.2animx.com/index-comic-name-%E9%A2%A8%E9%9B%B2%E5%85%A8%E9%9B%86-id-7212",
        );
        extr.fetch_chapters(&mut comic).unwrap();
        assert_eq!(670, comic.chapters.len());
        // let chapter1 = &mut comic.chapters[2];
        // extr.fetch_pages(chapter1).unwrap();
        // assert_eq!("風雲全集 第21卷", chapter1.title);
        // assert_eq!(29, chapter1.pages.len());
    }
}
