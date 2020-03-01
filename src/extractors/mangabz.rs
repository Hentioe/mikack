use super::*;
use reqwest::header::REFERER;

def_regex2![
    SCRIPT  => r#"var isVip = "[^"]+";(.+)reseturl"#
];

/// 对 www.mangabz.com 内容的抓取实现
/// 未来计划
/// - 支持简繁体切换
def_extractor! {
	state	=> [
		usable: true, pageable: true, searchable: true, https: false,
		favicon: "http://www.mangabz.com/favicon.ico"
	],
	tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("http://www.mangabz.com/manga-list-0-0-2-p{}/", page);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".mh-list > li",
            cover_dom       = "a > img",
            link_dom        = ".title > a",
            link_prefix     = "http://www.mangabz.com"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("http://www.mangabz.com/search?title={}", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".mh-list > li",
            cover_dom       = "a > img",
            link_dom        = ".title > a",
            link_prefix     = "http://www.mangabz.com"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = &"a.detail-list-form-item",
            link_prefix     = "http://www.mangabz.com"
        )?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;

        let script = match_content2!(&html, &*SCRIPT_RE)?;
        let wrap_code = wrap_code!(script, "
            var data = {
                title: MANGABZ_CTITLE,
                count: MANGABZ_IMAGE_COUNT,
                cid: MANGABZ_CID,
                mid: COMIC_MID,
                dt: MANGABZ_VIEWSIGN_DT,
                sign: MANGABZ_VIEWSIGN
            };
            data
        ", :end);
        let data = eval_as_obj(&wrap_code)?;
        let title = data.get_as_string("title")?;
        let total = data.get_as_int("count")?;
        let cid = *data.get_as_int("cid")?;
        let mid = *data.get_as_int("mid")?;
        let dt = data.get_as_string("dt")?.clone();
        let sign = data.get_as_string("sign")?.clone();

        let url = chapter.url.clone();
        chapter.set_title(title);
        let fetch = Box::new(move |current_page| {
            let page_url = format!(
                "http://www.mangabz.com/m{cid}/chapterimage.ashx?cid={cid}&page={page}&key=&_cid={cid}&_mid={mid}&_dt={dt}&_sign={sign}",
                cid = cid, page = current_page, mid = mid, dt = dt, sign = sign
            );
            let client = reqwest::blocking::Client::new();
            let page_html = client.get(&page_url).header(REFERER, &url).send()?.text()?;
            let wrap_code = format!("
                var data = {};
                data
            ", page_html);
            let mut pages = vec![];
            for (i, addr) in eval_value(&wrap_code)?.as_array()?.iter().enumerate() {
                pages.push(Page::new(current_page + i, addr.as_string()?));
            }

            Ok(pages)
        });

        Ok(ChapterPages::new(chapter, *total, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(12, comics.len());
        let comic1 = &mut Comic::from_link("地獄老師", "http://www.mangabz.com/1632bz/");
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(34, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("地獄老師 第1話", chapter1.title);
        assert_eq!(3, chapter1.pages.len());
        let comics = extr.search("地狱老师").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
