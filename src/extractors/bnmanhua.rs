use super::*;

def_regex2![
    DATA  => r#"<script>.+z_img=('[^']+')"#
];

/// 对 www.bnmanhua.com 内容的抓取实现
def_extractor! {
    status	=> [
        usable: true, pageable: true, searchable: true, https: true, pageable_search: true,
        favicon: "https://www.bnmanhua.com/favicon.ico"
    ],
    tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.bnmanhua.com/page/new/{}.html", page);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".plist02 > li",
            cover_dom       = "a > img",
            cover_attr      = "data-src",
            link_dom        = "a",
            link_prefix     = "https://www.bnmanhua.com",
            link_text_dom   = "p"
        )
    }

    fn paginated_search(&self, keywords: &str, page: u32) -> Result<Vec<Comic>> {
        let url = &if page == 1 {
            format!("https://www.bnmanhua.com/index.php?m=vod-search-wd-{}.html", keywords)
        } else {
            format!("https://www.bnmanhua.com/index.php?m=vod-search-pg-{}-wd-{}.html", page, keywords)
        };

        itemsgen2!(
            url             = url,
            parent_dom      = ".plist02 > li",
            cover_dom       = "a > img",
            cover_attr      = "data-src",
            link_dom        = "a",
            link_prefix     = "https://www.bnmanhua.com",
            link_text_dom   = "p"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = ".jslist01 > li > a",
            link_prefix     = "https://www.bnmanhua.com"
        )?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(document.dom_text(".mh_readtitle > strong")?);
        let data = match_content2!(&html, &*DATA_RE)?;

        let wrap_code = format!("
            var data = eval({data});
            data
        ", data = data);
        let data = eval_value(&wrap_code)?;
        let mut addresses = vec![];
        for path in data.as_array()? {
            let address = path.as_string()?;
            if !address.starts_with("http") {
                addresses.push(format!("https://img.yaoyaoliao.com/{}", address));
            } else {
                addresses.push(address.to_owned());
            }
        }

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(98, comics.len());
        let mut comic1 = Comic::new("天降魔女", "https://www.bnmanhua.com/comic/15195.html");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(67, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("天降魔女(02 买不买下他？)", chapter1.title);
        assert_eq!(32, chapter1.pages.len());
        let chapter2 = &mut Chapter::from_url("https://www.bnmanhua.com/comic/2393/1316201.html");
        extr.fetch_pages_unsafe(chapter2).unwrap();
        assert_eq!("进击的巨人(第128话叛徒)", chapter2.title);
        assert_eq!(42, chapter2.pages.len());
        let comics = extr.paginated_search("天降魔女", 1).unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
