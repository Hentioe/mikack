use super::*;

def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>>{
        let next = if page > 9 {
            "https://www.cartoonmad.com/endcm.0{}.html".to_string()
        }else{
            "https://www.cartoonmad.com/endcm.{}.html".to_string()
        };
        let url = urlgen![
            :first  => &"https://www.cartoonmad.com/endcm.01.html",
            :next   => &next,
            :page   => &page
        ];

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :href_prefix    => &"https://www.cartoonmad.com/",
            :target         => &r#"table[width="890"] td[colspan="2"] td[align="center"] > a"#,
            :encoding       => &BIG5
        ]
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(0).unwrap();
    assert_eq!(60, comics.len());

    // let mut comic = Comic::from_link("风云全集", "https://www.dm5.com/manhua-fengyunquanji/");
    // extr.fetch_chapters(&mut comic).unwrap();
    // assert_eq!(670, comic.chapters.len());

    // let chapter1 = &mut comic.chapters[27];
    // chapter1.title = "".to_string();
    // extr.fetch_pages(chapter1).unwrap();
    // assert_eq!("风云全集 第648卷 下", chapter1.title);
    // assert_eq!(14, chapter1.pages.len());
}
