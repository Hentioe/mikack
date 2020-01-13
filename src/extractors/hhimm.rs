use super::*;

def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("http://www.hhimm.com/comic/{}.html", page);

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :href_prefix    => &"http://www.hhimm.com",
            :target         => &r#"#list .cComicList > li > a"#
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"http://www.hhimm.com",
            :target         => &".cVolUl > li > a"
        ]?.attach_to(comic);

        Ok(())
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(30, comics.len());

    let mut comic = Comic::new("妖精的尾巴", "http://www.hhimm.com/manhua/2779.html");
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(597, comic.chapters.len());

    // let chapter1 = &mut comic.chapters[0];
    // chapter1.title = "".to_string();
    // extr.fetch_pages(chapter1).unwrap();
    // assert_eq!("龙珠 - 第519话 再见，龙珠", chapter1.title);
    // assert_eq!(19, chapter1.pages.len());
}
