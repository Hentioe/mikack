use super::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PageJson {
    #[serde(rename(deserialize = "listImg"))]
    list_img: Vec<String>,
}

def_regex2![
    SCRIPT  => r#"<script>(var did.+templatepath[^;]+;)</script>"#
];

/// 对 www.qimiaomh.com 内容的抓取实现
def_extractor! {
	status	=> [
		usable: true, pageable: true, searchable: true, https: true,
		favicon: "https://www.qimiaomh.com/favicon.ico"
	],
	tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.qimiaomh.com/list-1------updatetime--{}.html", page);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".classification",
            cover_dom       = "a > img",
            cover_attr      = "data-src",
            link_dom        = "h2 > a",
            link_prefix     = "https://www.qimiaomh.com"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.qimiaomh.com/action/Search?keyword={}", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".classification",
            cover_dom       = "a > img",
            cover_attr      = "data-src",
            link_dom        = "h2 > a",
            link_prefix     = "https://www.qimiaomh.com"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = ".comic-content .tit > a",
            link_prefix     = "https://www.qimiaomh.com"
        )?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(format!("{} {}",
            document.dom_text("h1.title")?,
            document.dom_text(".mCustomScrollBox ul > li:last-child > a")?
        ));
        let script = match_content2!(&html, &*SCRIPT_RE)?;

        let wrap_code = wrap_code!(script, "
            var data = {did: did, sid: sid};
            data
        ", :end);
        let data = eval_as_obj(&wrap_code)?;
        let did = data.get_as_int("did")?.clone();
        let sid = data.get_as_int("sid")?.clone();
        let json = get(&format!("https://www.qimiaomh.com/Action/Play/AjaxLoadImgUrl?did={}&sid={}", did, sid))?
            .json::<PageJson>()?;
        let addresses = json.list_img;

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(33, comics.len());
        let mut comic1 = Comic::new("玉猪龙", "https://www.qimiaomh.com/manhua/5955.html");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(48, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("玉猪龙 预告", chapter1.title);
        assert_eq!(7, chapter1.pages.len());
        let comics = extr.search("玉猪龙").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
