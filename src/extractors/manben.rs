use super::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ComicItem {
    #[serde(rename(deserialize = "Title"))]
    title: String,
    #[serde(rename(deserialize = "BigPic"))]
    big_pic: String,
    #[serde(rename(deserialize = "Url"))]
    url: String,
}

impl From<&ComicItem> for Comic {
    fn from(c: &ComicItem) -> Self {
        Self::from_index(
            &c.title,
            &format!("http://www.manben.com{}", &c.url),
            &c.big_pic,
        )
    }
}

def_regex2![
    SCRIPT  => r#"(eval.+\{\}\)\))"#
];

/// 对 www.manben.com 内容的抓取实现
def_extractor! {[usable: true, pageable: true, searchable: true],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let datetime = ""; // 时间字符串，例：Mon Feb 10 2020 23:32:51 GMT+0800 (中国标准时间)
        let url = format!("http://www.manben.com/mh-updated/pagerdata.ashx?d={}", datetime);
        let pageindex = &page.to_string();

        let mut params = HashMap::new();
        params.insert("t", "8");
        params.insert("pageindex", pageindex);
        params.insert("sc", "1");
        let client = Client::new();
        let comics = client
            .post(&url)
            .form(&params)
            .send()?
            .json::<Vec<ComicItem>>()?
            .iter()
            .map(|c: &ComicItem| { Comic::from(c) })
            .collect::<Vec<_>>();

        Ok(comics)
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("http://www.manben.com/search?title={}", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = r#"div[class^="bookList"] > .item"#,
            cover_dom       = ".book img",
            link_dom        = ".title > a",
            link_prefix     = "http://www.manben.com"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = &"#chapterlistload > .list > a",
            link_prefix     = &"http://www.manben.com",
        )?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(format!("{} {}",
            document.dom_text("#chapter")?,
            document.dom_text(".title-comicHeading")?
        ));
        let script = match_content2!(&html, &*SCRIPT_RE)?;

        let wrap_code = wrap_code!(script, "
            newImgs
        ", :end);
        let imgs = eval_value(&wrap_code)?;
        let mut addresses = vec![];
        for img in imgs.as_array()? {
            addresses.push(img.as_string()?.clone());
        }

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(10, comics.len());
        let comic1 = &mut Comic::from_link("琅琊榜", "http://www.manben.com/mh-langyabang/");
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(52, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[2];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("琅琊榜 第1回", chapter1.title);
        assert_eq!(46, chapter1.pages.len());
        let comics = extr.search("琅琊榜").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
