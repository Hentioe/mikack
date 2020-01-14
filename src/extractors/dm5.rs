use super::*;
use reqwest::header::REFERER;
use url::form_urlencoded;

def_regex![
    PARAMS_CODE_RE => r#"<script type="text/javascript">\s+var\s{1,}isVip\s{1,}=\s{1,}"False";(.+)\s+reseturl\(.+\);\s+</script>"#
];

def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = urlgen![
            :first  => &"http://www.dm5.com/manhua-rank/?t=1",
            :next   => &"http://www.dm5.com/manhua-rank/?t={}",
            :page   => &page
        ];

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :href_prefix    => &"https://www.dm5.com",
            :selector       => &"ul.mh-list.col3.top-cat > li .mh-item-detali > h2.title"
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"https://www.dm5.com",
            :selector       => &"#chapterlistload ul > li",
            :find           => &"a[title]"
        ]?.attach_to(comic);

        Ok(())
    }

    fn fetch_pages(&self, chapter: &mut Chapter) -> Result<()> {
        let resp = get(&chapter.url)?;
        let html = resp.text()?;
        let document = parse_document(&html);

        if chapter.title.is_empty() {
            chapter.title = format!("{} {}",
                document.dom_text(".title > span.right-arrow")?,
                document.dom_text(".title > span.right-arrow:last-child")?);
        }

        let page_count = document.dom_text("#chapterpager > a:last-child")?.parse::<i32>()?;

        let params_code = match_content![
            :text   => &html,
            :regex  => &*PARAMS_CODE_RE
        ];

        let warp_params_code = wrap_code!(params_code, r#"
            var params = {cid: DM5_CID, mid: COMIC_MID, dt: DM5_VIEWSIGN_DT, sign: DM5_VIEWSIGN};
            params
        "#, :end);
        let obj = eval_as_obj(&warp_params_code)?;
        let cid = obj.get_as_int("cid")?.to_string();
        let mid = obj.get_as_int("mid")?.to_string();
        let dt = obj.get_as_string("dt")?;
        let sign = obj.get_as_string("sign")?;

        let referer = &chapter.url.clone();
        let page_get = |url: &str| -> Result<reqwest::blocking::Response> {
            let client = reqwest::blocking::Client::new();
            let resp = client.get(url)
                .header(REFERER, referer)
                .send()?;
            Ok(resp)
        };

        for page in 1..page_count+1{
            let query_params: String = form_urlencoded::Serializer::new(String::new())
                .append_pair("cid", &cid)
                .append_pair("page", &page.to_string())
                .append_pair("_cid", &cid)
                .append_pair("_mid", &mid)
                .append_pair("_dt", &dt)
                .append_pair("_sign", &sign)
                .finish();

            let api_url = format!("{}chapterfun.ashx?{}", &chapter.url, query_params);
            let eval_code = page_get(&api_url)?.text()?;
            let wrap_eval_code = format!("var pages = {}; pages", eval_code);
            let eval_r = eval_value(&wrap_eval_code)?;
            let pages = eval_r.as_array()?;
            chapter.push_page(Page::new((page-1) as usize , pages[0].as_string()?))
        }

        Ok(())
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(297, comics.len());

    let mut comic = Comic::from_link("风云全集", "https://www.dm5.com/manhua-fengyunquanji/");
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(670, comic.chapters.len());

    let chapter1 = &mut comic.chapters[27];
    chapter1.title = "".to_string();
    extr.fetch_pages(chapter1).unwrap();
    assert_eq!("风云全集 第648卷 下", chapter1.title);
    assert_eq!(14, chapter1.pages.len());
}
