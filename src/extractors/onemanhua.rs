use super::*;

def_regex2![
    ENCRYPTED_DATA => r#"C_DATA='([^']+)'"#
];

/// 对 www.onemanhua.com 内容的抓取实现
def_extractor! {
    status	=> [
        usable: true, pageable: true, searchable: true, https: true,
        favicon: "https://www.onemanhua.com/favicon.png"
    ],
    tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.onemanhua.com/show?orderBy=update&page={}", page);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".fed-list-info > .fed-list-item",
            cover_dom       = "a.fed-list-pics",
            cover_attr      = "data-original",
            link_dom        = "a.fed-list-title",
            link_prefix     = "https://www.onemanhua.com"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.onemanhua.com/search?searchString={}", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".fed-deta-info",
            cover_dom       = "a.fed-list-pics",
            cover_attr      = "data-original",
            link_dom        = "h1 > a",
            link_prefix     = "https://www.onemanhua.com"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = ".all_data_list > ul > li > a",
            link_prefix     = "https://www.onemanhua.com"
        )?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;

        let encrypted_data = match_content2!(&html, &*ENCRYPTED_DATA_RE)?;
        let wrap_code = format!(
            "
                {crypto_lib}
                {runtime}
                eval(
                    __cdecrypt(
                        'JRUIFMVJDIWE569j',
                        CryptoJS.enc.Base64.parse('{encrypted_data}').toString(CryptoJS.enc.Utf8)
                    )
                );
                var data = {{ 
                    path: mh_info.imgpath,
                    domain: mh_info.domain,
                    start: mh_info.startimg,
                    total: mh_info.totalimg,
                    comic_name: mh_info.mhname,
                    chapter_name: mh_info.pagename
                }};
                data
            ",
            crypto_lib = include_str!("../../assets/lib/crypto-js.js"),
            runtime = include_str!("../../assets/runtime/onemanhua.js"),
            encrypted_data = encrypted_data
        );

        let data = eval_as_obj(&wrap_code)?;
        let title = format!("{} {}", data.get_as_string("comic_name")?,data.get_as_string("chapter_name")?);
        chapter.set_title(title);

        let start = *data.get_as_int("start")?;
        let total = *data.get_as_int("total")?;
        let domain = data.get_as_string("domain")?;
        let path = data.get_as_string("path")?;

        let mut addresses = vec![];
        for i in 0..total {
            let n = start + i;
            let fname = if n < 10 {
                format!("000{}", n)
            } else if n < 100 {
                format!("00{}", n)
            } else if n < 1000 {
                format!("0{}", n)
            } else {
                n.to_string()
            };
            let addr = format!("https://{domain}/comic/{path}{fname}.jpg",
                domain = domain, path = path, fname = fname
            );
            addresses.push(addr);
        }

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(30, comics.len());
        let mut comic1 = Comic::new("最后的召唤师", "https://www.onemanhua.com/12436/");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(418, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("最后的召唤师 第1话1 契约", chapter1.title);
        assert_eq!(15, chapter1.pages.len());
        let comics = extr.search("最后的召唤师").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
