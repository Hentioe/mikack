use super::*;

def_regex! [
    ENCODE_TEXT_RE => r#"var\s*chapterImages\s*=\s*"([^"]+)""#,
    PATH_RE        => r#"var\s*chapterPath\s*=\s*"([^"]+)""#
];

def_extractor! {[usable: true, searchable: false],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.manhuadui.com/list/riben/{}/", page);

        itemsgen![
            :entry      => Comic,
            :url        => &url,
            :target     => &"ul.list_con_li > li h3> a"
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :target         => &"ul.list_con_li > li > a",
            :href_prefix    => &"https://www.manhuadui.com",
            :sub_dom_text   => &"span.list_con_zj"
        ]?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        let comic_name = document.dom_text(".head_title > h1 > a")?;
        let chapter_num = document.dom_text(".head_title > h2")?;
        chapter.title = format!("{} {}", comic_name, chapter_num);

        let encode_text = match_content![
            :text   => &html,
            :regex  => &*ENCODE_TEXT_RE
        ];

        let runtime = include_str!("../../runtime/manhuadui.js");
        let wrap_code = wrap_code!(runtime, format!("
            var chapterImages =
            \"{}\";

            decrypt20180904(chapterImages)
        ", encode_text), :end);

        let path = match_content![
            :text   => &html,
            :regex  => &*PATH_RE
        ];

        let mut addresses = vec![];
        for fname in eval_value(&wrap_code)?.as_array()? {
            let address = format!("https://img01.eshanyao.com/{}/{}",path, fname.as_string()?);
            addresses.push(address);
        }

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(36, comics.len());

    let mut comic = Comic::from_link(
        "境界触发者",
        "https://www.manhuadui.com/manhua/jingjiechufazhe/",
    );
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(190, comic.chapters.len());

    let chapter1 = &mut Chapter::new(
        "",
        "https://www.manhuadui.com/manhua/jingjiechufazhe/435634.html",
        0,
    );
    extr.fetch_pages(chapter1).unwrap();
    assert_eq!("境界触发者 189话", chapter1.title);
    assert_eq!(20, chapter1.pages.len());
}
