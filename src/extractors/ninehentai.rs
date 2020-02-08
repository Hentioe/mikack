use super::*;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use std::str;

def_regex2![
    ID  => r#"https?://9hentai\.com/g/(\d+)"#
];

#[derive(Debug, Deserialize)]
struct SearchJson {
    results: Vec<Book>,
}

#[derive(Debug, Deserialize)]
struct GetBookJson {
    results: Book,
}

#[derive(Debug, Deserialize)]
struct Book {
    id: i64,
    image_server: String,
    title: String,
    total_page: i32,
}

impl Book {
    fn url(&self) -> String {
        format!("https://9hentai.com/g/{}/", self.id)
    }

    fn cover(&self) -> String {
        format!(
            "{image_server}{id}/cover-small.jpg",
            image_server = self.image_server,
            id = self.id
        )
    }
}

/// 对 9hentai.com 内容的抓取实现
def_extractor! {[usable: true, pageable: true, searchable: true],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let mut body = String::from(r#"{"search":{"text":"","page":"#);
        body.push_str(&page.to_string());
        body.push_str(r#","sort":0,"pages":{"range":[0,2000]},"tag":{"text":"","type":1,"tags":[],"items":{"included":[],"excluded":[]}}}}"#);
        let client = Client::new();
        let json = client
            .post("https://9hentai.com/api/getBook")
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()?
            .json::<SearchJson>()?;

        let mut comics = vec![];
        for result in json.results {
            comics.push(Comic::from_index(&result.title, &result.url(), &result.cover()))
        }

        Ok(comics)
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let mut body = String::from(r#"{"search":{"text":""#);
        body.push_str(&keywords);
        body.push_str(r#"","page":0,"sort":0,"pages":{"range":[0,2000]},"tag":{"text":"","type":1,"tags":[],"items":{"included":[],"excluded":[]}}}}"#);
        let client = Client::new();
        let json = client
            .post("https://9hentai.com/api/getBook")
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()?
            .json::<SearchJson>()?;

        let mut comics = vec![];
        for result in json.results {
            comics.push(Comic::from_index(&result.title, &result.url(), &result.cover()))
        }

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.chapters.push(Chapter::from_link(&comic.title, &comic.url));

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let id = match_content2!(&chapter.url, &*ID_RE)?;
        let mut body = String::from(r#"{"id":"#);
        body.push_str(&id);
        body.push_str("}");
        let client = Client::new();
        let json = client
            .post("https://9hentai.com/api/getBookByID")
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()?
            .json::<GetBookJson>()?;
        let book = json.results;
        chapter.set_title(book.title);
        let mut addresses = vec![];
        for i in 1..(book.total_page + 1) {
            addresses.push(format!("{}{}/{}.jpg", book.image_server, book.id, i));
        }
        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(18, comics.len());
        let mut comic1 = Comic::new(
            "(C97) [Takeritake Daishuukakusai (Echigoya Takeru)] Ore no Senki ♂ no Sei de Kyucyou ga Yabai | My Crest Makes The House Leader Crazy (Fire Emblem: Three Houses) (English) =TLL + mrwayne=",
            "https://9hentai.com/g/60726/"
        );
        extr.fetch_chapters(&mut comic1).unwrap();
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!(
            "(C97) [Takeritake Daishuukakusai (Echigoya Takeru)] Ore no Senki ♂ no Sei de Kyucyou ga Yabai | My Crest Makes The House Leader Crazy (Fire Emblem: Three Houses) (English) =TLL + mrwayne=",
            chapter1.title
        );
        assert_eq!(29, chapter1.pages.len());
        let comics = extr
            .search("Takeritake Daishuukakusai (Echigoya Takeru)")
            .unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
