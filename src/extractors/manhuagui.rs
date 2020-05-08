use super::*;

def_regex2![
    CTYPTO => r#"window\["\\x65\\x76\\x61\\x6c"\]\((.+)\)\s+</script>"#
];

def_extractor! {
    status	=> [
        usable: true, pageable: false, searchable: true, https: true,
        favicon: "https://www.manhuagui.com/favicon.ico"
    ],
    tags	=> [Chinese],

    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        let url = "https://www.manhuagui.com/update/";

        itemsgen2!(
            url             = &url,
            parent_dom      = ".latest-list > ul > li",
            cover_dom       = "a > img",
            cover_attrs     = &["data-src", "src"],
            link_dom        = ".ell > a",
            link_prefix     = "https://www.manhuagui.com"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.manhuagui.com/s/{}.html", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".book-result > ul > li",
            cover_dom       = ".bcover > img",
            cover_attrs     = &["data-src", "src"],
            link_dom        = "dt > a",
            link_prefix     = "https://www.manhuagui.com"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        let html = &get(&comic.url)?.text()?;
        let document = parse_document(&html);
        for (i, elem) in document.select(&parse_selector(r#"div[id^="chapter-list-"]"#)?).enumerate() {
            let selector =  GroupedItemsSelector {
                document: Rc::new(parse_document(&elem.html())),
                group_dom: "ul",
                items_dom: "li > a",
                items_title_attr: "title",
                items_url_prefix: "https://www.manhuagui.com",
                ..Default::default()
            };
            comic.chapters.append(&mut selector.gen()?.reversed_flatten(i));
        }

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let runtime = include_str!("../../assets/runtime/manhuagui.js");
        let crypty_code = match_content2!(&html, &*CTYPTO_RE)?;
        let wrap_code = format!("{}\n{}", &runtime, format!("
            DATA = null;
            SMH = {{
                imgData: function(data) {{
                    DATA = {{
                        cid: data.cid,
                        e: data.sl.e,
                        m: data.sl.m,
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
        // let cid = obj.get_as_int("cid")?.clone();
        let e = obj.get_as_int("e")?.clone();
        let m = obj.get_as_string("m")?.clone();
        let name = obj.get_as_string("name")?.clone();
        let path = obj.get_as_string("path")?.clone();
        let files = obj.get_as_array("files")?.clone();
        let total = files.len() as i32;

        chapter.set_title(name);
        let fetch = Box::new(move |current_page: usize| {
            let file = files[current_page - 1].as_string()?;
            let address = format!("https://i.hamreus.com{}{}?e={}&m={}", path, file, e, m);
            Ok(vec![Page::new(current_page - 1, address)])
        });

        Ok(ChapterPages::new(chapter, total, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert!(comics.len() > 0);

        let mut comic1 = Comic::from_link("食戟之灵", "https://www.manhuagui.com/comic/2863/");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(298, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("食戟之灵番外 贪吃2", chapter1.title);
        assert_eq!(6, chapter1.pages.len());
        let comics = extr.search("食戟之灵").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
