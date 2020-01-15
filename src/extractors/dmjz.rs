use super::*;

def_regex![
    CTYPTO_RE => r#"<script type="text/javascript">([\s\S]+)var res_type"#
];

def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = urlgen![
            :first  => &"https://manhua.dmzj.com/rank/",
            :next   => &"https://manhua.dmzj.com/rank/total-block-{}.shtml",
            :page   => &page
        ];

        itemsgen![
            :entry      => Comic,
            :url        => &url,
            :selector   => &".middleright-right > .middlerighter",
            :find       => &".title > a"
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"http://manhua.dmzj.com",
            :selector       => &".cartoon_online_border > ul > li"
        ]?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let code = match_content![
            :text   => &html,
            :regex  => &*CTYPTO_RE
        ];
        let wrap_code = format!("{}\n{}", &code, "
            var obj = {
                title: `${g_comic_name} ${g_chapter_name}`,
                pages: eval(pages)
            };
            obj
        ");
        let obj = eval_as_obj(&wrap_code)?;
        if chapter.title.is_empty(){
            chapter.title = obj.get_as_string("title")?.clone();
        }
        let mut addresses = vec![];
        for path in obj.get_as_array("pages")? {
            let address = format!("https://images.dmzj.com/{}", path.as_string()?);
            addresses.push(address);
        }

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(20, comics.len());

    let mut comic = Comic::from_link("极主夫道", "http://manhua.dmzj.com/jizhufudao/");
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(47, comic.chapters.len());

    let chapter1 = &mut comic.chapters[0];
    chapter1.title = "".to_string();
    extr.fetch_pages(chapter1).unwrap();
    assert_eq!("极主夫道 第01话", chapter1.title);
    assert_eq!(16, chapter1.pages.len());
}
