use super::*;

def_exctractor! {
    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        Ok(vec![])
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.push_chapter(Chapter::from_link(comic.title.clone(), comic.url.clone()));

        Ok(())
    }

    // fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
    //     let html = get(&chapter.url)?.text()?;
    //     let document = parse_document(&html);
    //     Ok(ChapterPages::full(chapter, addresses))
    // }
}

#[test]
fn test_extr() {
    // let extr = new_extr();
    // let comics = extr.index(1).unwrap();
    // assert_eq!(30, comics.len());

    // let chapter1 = &mut Chapter::from_link("", "https://www.luscious.net/albums/teitoku-wa-semai-toko-suki-kantai-collection-kanco_363520/");
    // extr.fetch_pages(chapter1).unwrap();
    // assert_eq!("Teitoku wa Semai Toko Suki (Kantai Collection -KanColle-) [English]", chapter1.title);
    // assert_eq!(25, chapter1.pages.len());
}
