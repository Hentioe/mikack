use super::*;
use serde::Deserialize;

def_regex2![
    ID      => r#"https?://www\.tvbsmh\.com/comic-([^-]+)-.+"#,
    NAME    => r#"https?://www\.tvbsmh\.com/comic-[^-]+-([^?]+)"#
];

#[derive(Debug, Deserialize)]
struct ChapterItem {
    chapter_id: i64,
    title: String,
}

#[derive(Debug, Deserialize)]
struct SystemItem {
    system: ChapterItem,
}

#[derive(Debug, Deserialize)]
struct RepoJson {
    msg: Vec<SystemItem>,
}

impl ChapterItem {
    fn url(&self, comic_id: &str, comic_name: &str) -> String {
        format!(
            "https://www.tvbsmh.com/series-{comic_id}-{chapter_id}-1-{comic_name}",
            comic_id = comic_id,
            chapter_id = self.chapter_id,
            comic_name = comic_name
        )
    }
}

/// 对 www.tvbsmh.com 内容的抓取实现
def_extractor! {
    status	=> [
        usable: true, pageable: false, searchable: true, https: true,
        favicon: "https://www.tvbsmh.com/favicon.ico"
    ],
    tags	=> [Chinese],

    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        let url = "https://www.tvbsmh.com/comiclist/comiclistupdate";

        itemsgen2!(
            url             = url,
            parent_dom      = ".searchpage1 .list > li",
            cover_dom       = ".img > a > img",
            cover_attr      = "data-original",
            link_dom        = ".ti > a",
            link_prefix     = "https://www.tvbsmh.com"
        )
    }

    fn paginated_search(&self, keywords: &str, page: u32) -> Result<Vec<Comic>> {
        let url = &format!("https://www.tvbsmh.com/search?searhword={}&page={}", keywords, page);

        itemsgen2!(
            url             = url,
            parent_dom      = ".searchpage .list > li",
            cover_dom       = ".img > a > img",
            link_dom        = ".ti > a",
            link_prefix     = "https://www.tvbsmh.com"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        let id = &match_content2!(&comic.url, &*ID_RE)?;
        let name = &match_content2!(&comic.url, &*NAME_RE)?;
        let client = Client::new();
        let mut params: HashMap<_, &str> = HashMap::new();
        params.insert("cartoon_id", id);
        params.insert("order_by", "1");
        params.insert("chapter_type", "1");
        let json = client
            .post("https://www.tvbsmh.com/comicinfo-ajaxgetchapter.html")
            .header("x-requested-with", "XMLHttpRequest")
            .form(&params)
            .send()?
            .json::<RepoJson>()?;

        for (i, item) in json.msg.iter().rev().map(|s| &s.system ).enumerate() {
            comic.push_chapter(
                Chapter::new(&item.title, item.url(&id, &name), i as u32)
            );
        }

        Ok(())
    }

    // fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
    //     let html = get(&chapter.url)?.text()?;
    //     let document = parse_document(&html);
    // }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert!(comics.len() > 0);
        let comic1 = &mut Comic::new("三國誌異", "https://www.tvbsmh.com/comic-noczp-三國誌異");
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(64, comic1.chapters.len());
        // let chapter1 = &mut comic1.chapters[0];
        // extr.fetch_pages_unsafe(chapter1).unwrap();
        // assert_eq!("三國誌異 第1回", chapter1.title);
        // assert_eq!(25, chapter1.pages.len());
        let comics = extr.paginated_search("三國誌異", 1).unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
