use super::*;

def_regex![
    DECRYPT_RE => r#"<script type="text/javascript">[\s\n]+(eval.+)[\s\n]+</script>"#
];

def_extractor! {[usable: true, searchable: false],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let next_page = page - 1;
        let url = urlgen![
            :first  => &"https://www.177mh.net/wanjie/index.html",
            :next   => &"https://www.177mh.net/wanjie/index_{}.html",
            :page   => &next_page
        ];

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :href_prefix    => &"https://www.177mh.net",
            :target         => &r#".ar_list_co > ul > li > span > a"#
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"https://www.177mh.net",
            :target         => &"ul.ar_list_col > li > a"
        ]?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.title = document.dom_text("#tab_srv + h1 > a")?;
        let decrypt_code = match_content![
            :text   => &html,
            :regex  => &*DECRYPT_RE
        ];

        let wrap_code = wrap_code!(&decrypt_code, "
            var data = {msg: msg, img_s: img_s, link_z: link_z};
            data
        ", :end);
        let obj = eval_as_obj(&wrap_code)?;
        let img_s = obj.get_as_int("img_s")?;
        let msg = obj.get_as_string("msg")?;
        let link_z = obj.get_as_string("link_z")?;

        let wrap_img_qianzso_params_code = format!("
            var coid = /\\/(\\d+\\/\\d+)\\.html/.exec('{location}');
            var coid_num = /\\d+\\/(\\d+)/.exec(coid)[1];
            var cid = /\\/colist_(\\d+)\\.html/.exec('{link_z}')[1];
            var data = {{ coid_num: coid_num, cid: cid }};
            data
        ", location=&chapter.url, link_z=&link_z);

        let img_qianzso_params = eval_as_obj(&wrap_img_qianzso_params_code)?;
        let coid_num = img_qianzso_params.get_as_string("coid_num")?;
        let cid = img_qianzso_params.get_as_string("cid")?;
        let img_qianzso_url = format!(
            "https://css.gdbyhtl.net/img_v1/cn_svr.aspx?s={}&cid={}&coid={}",
            img_s, cid, coid_num
        );
        let img_qianzso_code = get(&img_qianzso_url)?.text()?;
        let wrap_img_qianzso_code = wrap_code!(&img_qianzso_code, format!("
            img_qianzso[{}]
        ", img_s), :end);
        let img_qianz = eval_as::<String>(&wrap_img_qianzso_code)?;

        let mut addresses = vec![];
        for file in msg.split("|") {
            let address = format!("{}{}", img_qianz, file);
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

    let mut comic = Comic::new("火影忍者", "https://www.177mh.net/colist_78825.html");
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(517, comic.chapters.len());

    let chapter1 = &mut comic.chapters[516];
    chapter1.title = "".to_string();
    extr.fetch_pages(chapter1).unwrap();
    assert_eq!("火影忍者 外传_满月", chapter1.title);
    assert_eq!(44, chapter1.pages.len());
}
