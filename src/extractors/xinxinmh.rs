use super::*;

def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let next_page = page - 1;
        let url = urlgen![
            :first  => &"https://www.177mh.net/wanjie/index.html",
            :next   => &"https://www.177mh.net/wanjie/index_{}.html",
            :page   => &next_page
        ];

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :href_prefix    => &"https://www.177mh.net",
            :target         => &r#".ar_list_co > ul > li > span > a"#
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"https://www.177mh.net",
            :target         => &"ul.ar_list_col > li > a"
        ]?.attach_to(comic);

        Ok(())
    }

    // fn fetch_pages(&self, chapter: &mut Chapter) -> Result<()> {

    //     Ok(())
    // }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(20, comics.len());

    let mut comic = Comic::new("火影忍者", "https://www.177mh.net/colist_78825.html");
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(517, comic.chapters.len());

    // let chapter1 = &mut comic.chapters[2];
    // chapter1.title = "".to_string();
    // extr.fetch_pages(chapter1).unwrap();
    // assert_eq!("妖精的尾巴 543集", chapter1.title);
    // assert_eq!(22, chapter1.pages.len());
}
