use super::*;

def_regex![
    CTYPTO_RE => r#"window\["\\x65\\x76\\x61\\x6c"\]\((.+)\)\s+</script>"#
];

def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = urlgen![
            :first  => &"https://www.manhuagui.com/list/",
            :next   => &"https://www.manhuagui.com/list/index_p{}.html",
            :page   => &page
        ];

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :href_prefix    => &"https://www.manhuagui.com",
            :target         => &"#contList .ell > a"
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"https://www.manhuagui.com",
            :target       => &"li > a.status0"
        ]?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let runtime = include_str!("../../runtime/manhuagui.js");
        let crypty_code = match_content![
            :text   => &html,
            :regex  => &*CTYPTO_RE
        ];
        let wrap_code = format!("{}\n{}", &runtime, format!("
            DATA = null;
            SMH = {{
                imgData: function(data) {{
                    DATA = {{
                        cid: data.cid,
                        md5: data.sl.md5,
                        name: `${{data.bname + data.cname}}`,
                        path: data.path,
                        files: data.files
                    }};
                }}
            }};

            try {{ eval({}) }} catch (error) {{}}
            DATA
        ", &crypty_code));
        let obj = eval_as_obj(&wrap_code)?;
        let cid = obj.get_as_int("cid")?.clone();
        let md5 = obj.get_as_string("md5")?.clone();
        let name = obj.get_as_string("name")?.clone();
        let path = obj.get_as_string("path")?.clone();
        let files = obj.get_as_array("files")?.clone();
        let total = files.len() as i32;
        chapter.title = name;

        let fetch = Box::new(move |current_page: usize| {
            let file = files[current_page - 1].as_string()?;
            let address = format!("https://i.hamreus.com{}{}?cid={}&md5={}", path, file, cid, md5);
            Ok(vec![Page::new(current_page - 1, address)])
        });

        Ok(ChapterPages::new(chapter, total, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(42, comics.len());

    let mut comic = Comic::from_link("火影忍者", "https://www.manhuagui.com/comic/4681/");
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(96, comic.chapters.len());

    let chapter1 = &mut comic.chapters[0];
    extr.fetch_pages(chapter1).unwrap();
    assert_eq!("火影忍者第01卷", chapter1.title);
    assert_eq!(190, chapter1.pages.len());
}
