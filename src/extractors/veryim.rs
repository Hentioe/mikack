use super::*;

/// 对 comic.veryim.com 内容的抓取实现
/// 注意： 由于上游问题 fetch_pages 并未实现，需进一步考察上游稳定性
def_exctractor! {
    fn is_usable(&self) -> bool { false }
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("http://comic.veryim.com/allhit/{}.html", page);

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :href_prefix    => &"http://comic.veryim.com",
            :target         => &r#"ul.grid-row.clearfix > li > p > a"#
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"http://comic.veryim.com",
            :target         => &".chapters > ul.clearfix > li > a"
        ]?.reversed_attach_to(comic);

        Ok(())
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let mut comics = extr.index(1).unwrap();
    assert_eq!(28, comics.len());

    let mut comic = &mut comics[0];
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(519, comic.chapters.len());

    // let chapter1 = &mut comic.chapters[0];
    // chapter1.title = "".to_string();
    // extr.fetch_pages(chapter1).unwrap();
    // assert_eq!("龙珠 - 第519话 再见，龙珠", chapter1.title);
    // assert_eq!(19, chapter1.pages.len());
}
