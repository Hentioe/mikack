use super::*;

def_regex2![
    DECRYPT => r#"<script type="text/javascript">[\s\n]+(eval.+)[\s\n]+</script>"#
];

/// 对 www.177mh.net 内容的抓取实现
def_extractor! {
	state	=> [
		usable: true, pageable: true, searchable: true, https: true,
		favicon: "https://www.177mh.net/favicon.ico"
	],
	tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = urlgen2!(page - 1,
            first   = "https://www.177mh.net/lianzai/index.html",
            next    = "https://www.177mh.net/lianzai/index_{}.html"
        );

        itemsgen2!(
            url             = &url,
            parent_dom      = ".ar_list_co > ul > li",
            cover_dom       = "a > img",
            link_dom        = "span > a",
            link_prefix     = "https://www.177mh.net"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://so.177mh.net/k.php?k={}", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".so_head + ul > dl",
            cover_dom       = "a > img",
            link_dom        = "h1 > a"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = "ul.ar_list_col > li > a",
            link_prefix     = "https://www.177mh.net"
        )?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.title = document.dom_text("#tab_srv + h1 > a")?;
        let decrypt_code = match_content2!(&html, &*DECRYPT_RE)?;

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
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(20, comics.len());
        let mut comic1 = Comic::new("祝福之钟", "https://www.177mh.net/colist_104931.html");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(21, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages(chapter1).unwrap();
        assert_eq!("祝福之钟 第001话", chapter1.title);
        assert_eq!(24, chapter1.pages.len());
        let comics = extr.search("祝福之钟").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
