use super::*;
use serde::Deserialize;

def_regex2![
    COMIC_ID  => r#"https?://www\.comico\.com\.tw/(\d+)"#
];

#[derive(Debug, Deserialize)]
struct ComicJson {
    result: ComicResult,
}

#[derive(Debug, Deserialize)]
struct ComicResult {
    list: Vec<ComicArticle>,
}

#[derive(Debug, Deserialize)]
struct ComicArticle {
    article_url: String,
    article_title: String,
    img_url: String,
}

#[derive(Debug, Deserialize)]
struct ChapterJson {
    result: ChapterResult,
}

#[derive(Debug, Deserialize)]
struct ChapterResult {
    list: Vec<ChapterArticle>,
}

#[derive(Debug, Deserialize)]
struct ChapterArticle {
    #[serde(rename(deserialize = "articleDetailUrl"))]
    article_detail_url: String,
    subtitle: String,
}

/// 对 www.comico.com.tw 内容的抓取实现
def_extractor! {
    status	=> [
		usable: true, searchable: true, pageable: true, https: false,
		favicon: "http://www.comico.com.tw/favicon.ico"
	],
    tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = "http://www.comico.com.tw/challenge/updateList.nhn?order=update";

        let client = Client::new();
        let mut params = HashMap::new();
        params.insert("page", page.to_string());
        let json = client
            .post(url)
            .form(&params)
            .send()?
            .json::<ComicJson>()?;
        let mut comics = vec![];
        for article in json.result.list {
            comics.push(Comic::from_index(&article.article_title, &article.article_url, &article.img_url));
        }

        Ok(comics)
    }

    fn search(&self, keyworkds: &str) -> Result<Vec<Comic>> {
        let url = format!("http://www.comico.com.tw/search/index.nhn?searchWord={}", keyworkds);

        itemsgen2!(
            url             = &url,
            parent_dom      = r#"div[class^="list-article"] li[class$="_item"]"#,
            cover_dom       = "img.list-article02__cover-img",
            link_dom        = "a.list-article02__item-inner",
            link_text_attr  = "title"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        let id = match_content2!(&comic.url, &*COMIC_ID_RE)?;
        let url = "http://www.comico.com.tw/api/getArticleListAll.nhn";

        let client = Client::new();
        let mut params = HashMap::new();
        params.insert("titleNo", id);
        let json = client
            .post(url)
            .form(&params)
            .send()?
            .json::<ChapterJson>()?;
        json.result
            .list
            .iter()
            .map(|article| {
                Chapter::from_link(article.subtitle.clone(), article.article_detail_url.clone())
            })
            .collect::<Vec<_>>()
            .reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(document.dom_text(".comico-global-header__page-title > p")?.replace("  ", " "));
        let addresses = document.dom_attrs(".comic-image > img", "src")?;

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(20, comics.len());
        let comic1 = &mut Comic::from_link("三毛與捲毛第二季", "http://www.comico.com.tw/3711/");
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(12, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("三毛與捲毛第二季01 煩人警察", chapter1.title);
        assert_eq!(20, chapter1.pages.len());
        let comics = extr.search("三毛與捲毛第二季").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
