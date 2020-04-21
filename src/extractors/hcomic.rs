use super::*;
use reqwest::{blocking::Client, header::LOCATION, redirect::Policy};

/// 对 ahmog.com 内容的抓取实现
/// TODO: 从 c-upp.com 修正到最新的域名
def_extractor! {
    status	=> [
        usable: true, pageable: true, searchable: true, https: true, pageable_search: true,
        favicon: "https://ahmog.com/favicon.ico"
    ],
    tags	=> [English, Japanese, Chinese, NSFW],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://ahmog.com/search/1919814/{}/", page - 1);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".file_list > li",
            cover_dom       = "img.lazy",
            link_dom        = "a[title]",
            link_prefix     = "https://ahmog.com",
            link_text_attr  = "title"
        )
    }

    fn paginated_search(&self, keywords: &str, page: u32) -> Result<Vec<Comic>> {
        let url = "https://ahmog.com/search/";

        let mut params = HashMap::new();
        params.insert("show", "title,titleen,tags");
        params.insert("keyboard", keywords);

        let client = Client::builder()
            .redirect(Policy::none())
            .build()?;
        let resp = client
            .post(url)
            .form(&params)
            .send()?;
        if resp.status().is_redirection() {
            let redirected_url = resp.headers().get(LOCATION).ok_or(err_msg("No redirect address"))?.to_str()?;
            let paginated_search_url = &format!("https://ahmog.com{}{}/", redirected_url, page - 1);
            let html = get(paginated_search_url)?.text()?;
            itemsgen2!(
                html            = &html,
                parent_dom      = ".file_list > li",
                cover_dom       = "img.lazy",
                link_dom        = "a[title]",
                link_prefix     = "https://ahmog.com",
                link_text_attr  = "title"
            )

        } else {
            Err(err_msg("No redirect page"))
        }
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.chapters.push(Chapter::from_link(&comic.title, &comic.url));

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(document.dom_attr(".image > a > img", "alt")?);
        let addresses = document
            .dom_attrs(".img_list .image > img", "src")?
            .iter()
            .map(|addr| {
                addr.replace("pic.comicstatic.icu", "img.comicstatic.icu").clone()
            })
            .collect::<Vec<_>>();
        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(50, comics.len());
        let comic1_title =
            "(C97) [Sonotaozey (Yukataro)] ANDO/OSHIDA,motto Nakayoku! (Girls und Panzer) [Chinese] [沒有漢化]";
        let mut comic1 = Comic::new(comic1_title, "https://ahmog.com/cn/s/316652/");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(1, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!(comic1_title, chapter1.title);
        assert_eq!(35, chapter1.pages.len());
        let comics = extr
            .paginated_search("motto Nakayoku!", 1)
            .unwrap();
        assert!(comics.len() > 0);
        assert!(comics[0].title.contains(&comic1.title));
        assert_eq!(comics[0].url, comic1.url);
    }
}
