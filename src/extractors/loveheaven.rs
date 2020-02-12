use super::*;
use serde::{Deserialize, Serialize};

def_regex2![
    ONCLICK_PATH    => r#"window\.location='([^']+)'"#
];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchJson {
    data: Vec<SearchDataItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchDataItem {
    image: String,
    primary: String,
    onclick: String,
}

impl SearchDataItem {
    fn url(&self) -> Result<String> {
        let path = match_content2!(&self.onclick, &*ONCLICK_PATH_RE)?;

        Ok(format!("https://loveheaven.net{}", path))
    }
}

impl From<&SearchDataItem> for Comic {
    fn from(item: &SearchDataItem) -> Self {
        if let Ok(url) = item.url() {
            Comic::from_index(&item.primary, &url, &item.image)
        } else {
            Comic::from_index(&item.primary, &item.onclick, &item.image)
        }
    }
}

/// 对 loveheaven.net 内容的抓取实现
def_extractor! {[usable: true, searchable: false],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!(
            "https://loveheaven.net/manga-list.html?listType=pagination&page={}&sort=last_update&sort_type=DESC",
            page
        );

        itemsgen2!(
            url             = &url,
            parent_dom      = ".row > .row-list",
            cover_dom       = ".img-thumb",
            link_dom        = ".media-heading > a",
            link_prefix     = "https://loveheaven.net/"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://loveheaven.net/app/manga/controllers/search.single.php?q={}", keywords);

        let comics = get(&url)?.json::<Vec<SearchJson>>()?
            .iter()
            .next()
            .ok_or(err_msg("Inapplicable response structure"))?
            .data
            .iter()
            .map(|item: &SearchDataItem| {
                Comic::from(item)
            })
            .collect::<Vec<_>>();

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = r#"td > a.chapter"#,
            link_prefix     = "https://loveheaven.net/"
        )?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.title = document.dom_attr(r#"li[itemprop="itemListElement"]:last-child > a"#, "title")?;
        let addresses = document.dom_attrs(".chapter-img", "src")?;
        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(20, comics.len());
        let comic1 = &mut Comic::from_link(
            "Ichinichi Gaishutsuroku Hanchou - Raw",
            "https://loveheaven.net/manga-ichinichi-gaishutsuroku-hanchou-raw.html",
        );
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(52, comic1.chapters.len());
        let chapter1 = &mut Chapter::from_link(
            "",
            "https://loveheaven.net/read-ichinichi-gaishutsuroku-hanchou-raw-chapter-64.html",
        );
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!(
            "Ichinichi Gaishutsuroku Hanchou - Raw chap 64",
            chapter1.title
        );
        assert_eq!(19, chapter1.pages.len());
        let comics = extr.search("Ichinichi Gaishutsuroku Hanchou").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
