use super::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PageJson {
    #[serde(rename(deserialize = "Code"))]
    code: String,
}

def_regex2![
    COVER   => r#"background-image: url\((.+)\)"#,
    SCRIPT  => r#"<script type="text/javascript">\s+(var imgDomain[\s\S]+var bqimg[^<]+)</script>"#
];

/// 对 www.tohomh123.com 内容的抓取实现
def_extractor! {
	state	=> [usable: true, pageable: true, searchable: true],
	tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.tohomh123.com/f-1-1-----updatetime--{}.html", page);

        let comics = itemsgen2!(
            url             = &url,
            parent_dom      = ".mh-list .mh-item",
            cover_dom       = ".mh-cover",
            cover_attr      = "style",
            link_dom        = "a",
            link_prefix     = "https://www.tohomh123.com",
            link_text_attr  = "title",
        )?
        .iter_mut()
        .map(|comic: &mut Comic| {
            if let Ok(cover) = match_content2!(&comic.cover, &*COVER_RE) {
                comic.cover = cover.to_string();
            }
            comic.clone()
        })
        .collect::<Vec<_>>();

        Ok(comics)
    }

    fn search(&self, keyworkds: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.tohomh123.com/action/Search?keyword={}", keyworkds);

        itemsgen2!(
            url             = &url,
            parent_dom      = r#".am-thumbnails.list"#,
            cover_dom       = ".container > img",
            link_dom        = ".am-thumbnail > a",
            link_prefix     = "https://www.tohomh123.com",
            link_text_attr  = "title"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = r#"ul[id^="detail-list-select"] > li > a"#,
            link_prefix     = "https://www.tohomh123.com"
        )?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(document.dom_text("h1.title")?);
        let script = match_content2!(&html, &*SCRIPT_RE)?;

        let wrap_code = wrap_code!(script, "
            var data = {did: did, sid: sid, pcount: pcount};
            data
        ", :end);
        let data = eval_as_obj(&wrap_code)?;
        let did = data.get_as_int("did")?.clone();
        let sid = data.get_as_int("sid")?.clone();
        let total = data.get_as_int("pcount")?.clone();
        let fetch = Box::new(move |current_page: usize| {
            let page_url = format!(
                "https://www.tohomh123.com/action/play/read?did={did}&sid={sid}&iid={iid}",
                did = did, sid = sid, iid = current_page
            );
            let json = get(&page_url)?.json::<PageJson>()?;
            Ok(vec![Page::new(current_page - 1, json.code)])
        });

        Ok(ChapterPages::new(chapter, total, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(35, comics.len());
        let mut comic1 = Comic::new("光之子", "https://www.tohomh123.com/guangzhizi/");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(42, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[20];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("光之子光系魔法-永恒", chapter1.title);
        assert_eq!(6, chapter1.pages.len());
        let comics = extr.search("光之子").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
