use super::*;
use reqwest::header::REFERER;
use url::form_urlencoded;

def_regex2![
    PARAMS_CODE => r#"<script type="text/javascript">\s+var\s{1,}isVip\s{1,}=\s{1,}"False";(.+)\s+reseturl\(.+\);\s+</script>"#,
    COVER_URL   => r#"background-image: url\((.+)\)"#
];

def_extractor! {
    status	=> [
        usable: true, pageable: false, searchable: true, https: true,
        favicon: "https://www.dm5.com/favicon.ico"
    ],
    tags	=> [Chinese],

    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        let url = "https://www.dm5.com/manhua-new/";

        let mut comics = itemsgen2!(
            url         = url,
            parent_dom  = "li > .mh-item",
            cover_dom   = ".mh-cover",
            cover_attr  = "style",
            link_dom    = "h2.title > a",
            link_prefix = "https://www.dm5.com"
        )?;

        comics.iter_mut().for_each(|comic: &mut Comic| {
            if let Ok(cover) = match_content2!(&comic.cover, &*COVER_URL_RE) {
                comic.cover = cover.clone()
            }
        });

        Ok(comics)
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.dm5.com/search?title={}", keywords);
        let html = get(&url)?.text()?;
        let document = parse_document(&html);
        let mut comics = vec![];

        let banner_cover = document.dom_attr(".banner_detail_form > .cover > img", "src")?;
        let banner_title = document.dom_text(".banner_detail_form .title > a")?;
        let banner_url = format!("https://www.dm5.com{}", document.dom_attr(".banner_detail_form .title > a", "href")?);

        comics.push(Comic::from_index(banner_title, banner_url, banner_cover));

        comics.append(&mut itemsgen2!(
            html        = &html,
            parent_dom  = "li > .mh-item",
            cover_dom   = ".mh-cover",
            cover_attr  = "style",
            link_dom    = "h2.title > a",
            link_prefix = "https://www.dm5.com"
        )?);

        comics.iter_mut().for_each(|comic: &mut Comic| {
            if let Ok(cover) = match_content2!(&comic.cover, &*COVER_URL_RE) {
                comic.cover = cover.clone()
            }
        });

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        let html = &get(&comic.url)?.text()?;
        if html.contains("view-win-list") {
            itemsgen2!(
                html        = html,
                target_dom  = "#chapterlistload ul > li > a[title]",
                link_prefix = "https://www.dm5.com"
            )?.reversed_attach_to(comic);
        } else {
            itemsgen2!(
                html            = html,
                target_dom      = "#chapterlistload ul > li > a[title]",
                link_text_dom   = ".info > .title",
                link_prefix     = "https://www.dm5.com",
                ignore_contains = ".detail-lock" // 排除付费章节
            )?.attach_to(comic);
        }

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let url = chapter.url.clone();
        let resp = get(&url)?;
        let html = resp.text()?;
        let document = parse_document(&html);

       chapter.title = format!("{} {}",
            document.dom_text(".title > span.right-arrow")?,
            document.dom_text(".title > span.right-arrow:last-child")?);

        if document.dom_count("#barChapter")? > 0 { // 包含全部图片资源（无需翻页）
            let addresses = document.dom_attrs("#barChapter > img", "data-src")?;
            Ok(ChapterPages::full(chapter, addresses))
        } else {
            let page_count =
                if document.dom_count("#chapterpager > a:last-child")? == 0 {
                    1
                }
                else {
                    document.dom_text("#chapterpager > a:last-child")?.parse::<i32>()?
                };
            let params_code = match_content2!(&html, &*PARAMS_CODE_RE)?;

            let warp_params_code = wrap_code!(params_code, r#"
                var params = {cid: DM5_CID, mid: COMIC_MID, dt: DM5_VIEWSIGN_DT, sign: DM5_VIEWSIGN};
                params
            "#, :end);
            let obj = eval_as_obj(&warp_params_code)?;
            let cid = obj.get_as_int("cid")?.to_string();
            let mid = obj.get_as_int("mid")?.to_string();
            let fetch_url = move |current_page: usize| -> Result<String> {
                let query_params: String = form_urlencoded::Serializer::new(String::new())
                .append_pair("cid", &cid)
                .append_pair("page", &current_page.to_string())
                .append_pair("_cid", &cid)
                .append_pair("_mid", &mid)
                .append_pair("_dt", obj.get_as_string("dt")?)
                .append_pair("_sign", obj.get_as_string("sign")?)
                .finish();

                let api_url = format!("{}chapterfun.ashx?{}", url, query_params);
                let client = reqwest::blocking::Client::new();
                let eval_code = client.get(&api_url).header(REFERER, &url).send()?.text()?;
                let wrap_eval_code = format!("var pages = {}; pages", eval_code);
                let eval_r = eval_value(&wrap_eval_code)?;
                Ok(eval_r.as_array()?[0].as_string()?.clone())
            };

            let mut first_page_addresses = vec![];
            first_page_addresses.push(fetch_url(1)?);

            let fetch = Box::new(move |current_page: usize| {
                let address = fetch_url(current_page)?;
                Ok(vec![Page::new(current_page - 1, address)])
            });

            Ok(ChapterPages::new(chapter, page_count, first_page_addresses, fetch))
        }
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert!(0 < comics.len());
        let mut comic1 = Comic::from_link("风云全集", "https://www.dm5.com/manhua-fengyunquanji/");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(670, comic1.chapters.len());
        let mut comic2 = Comic::from_url("https://www.dm5.com/manhua-badaogushao-qingqingliao/");
        extr.fetch_chapters(&mut comic2).unwrap();
        assert_eq!(11, comic2.chapters.len());
        let chapter1 = &mut comic1.chapters[642];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("风云全集 第648卷 下", chapter1.title);
        assert_eq!(14, chapter1.pages.len());
        let chapter2 = &mut Chapter::from_url("https://www.dm5.com/m893482/");
        extr.fetch_pages_unsafe(chapter2).unwrap();
        assert_eq!("神武天尊 第1回", chapter2.title);
        assert_eq!(39, chapter2.pages.len());
        let chapter3 = &mut Chapter::from_url("https://www.dm5.com/m896094/");
        extr.fetch_pages_unsafe(chapter3).unwrap();
        assert_eq!("公主链接小四格 第1话 为了谁？", chapter3.title);
        assert_eq!(1, chapter3.pages.len());
        let mut comic3 =
            Comic::from_url("https://www.dm5.com/manhua-yitaishuangbao-guaigemamidaihuijia/");
        extr.fetch_chapters(&mut comic3).unwrap();
        assert_eq!(34, comic3.chapters.len());
        let comics = extr.search("风云全集").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
