use super::*;
use reqwest::blocking::Client;

def_regex2![
    COVER => r#"background: url("([^"]+)")"#
];

/// 对 c-upp.com 内容的抓取实现
def_extractor! {[usable: true, pageable: true, searchable: true],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://c-upp.com/search/1919814/{}/", page - 1);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".file_list > li",
            cover_dom       = "img.lazy",
            link_dom        = "a[title]",
            link_prefix     = "https://c-upp.com",
            link_text_attr  = "title"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = "https://c-upp.com/search/";

        let mut params = HashMap::new();
        params.insert("show", "title,titleen,tags");
        params.insert("keyboard", keywords);

        let client = Client::new();
        let html = client
            .post(url)
            .form(&params)
            .send()?
            .text()?;

        itemsgen2!(
            html            = &html,
            parent_dom      = ".file_list > li",
            cover_dom       = "img.lazy",
            link_dom        = "a[title]",
            link_prefix     = "https://c-upp.com",
            link_text_attr  = "title"
        )
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
        let mut comic1 = Comic::new(
            "(C97) [Sonotaozey (Yukataro)] ANDO/OSHIDA,motto Nakayoku! (Girls und Panzer)",
            "https://c-upp.com/ja/s/315668/",
        );
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(1, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!(
            "(C97) [Sonotaozey (Yukataro)] ANDO/OSHIDA,motto Nakayoku! (Girls und Panzer)",
            chapter1.title
        );
        assert_eq!(34, chapter1.pages.len());
        let comics = extr.search("あんおし、もっとなかよく").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
