use super::*;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Item {
    #[serde(rename(deserialize = "Title"))]
    title: String,
    #[serde(rename(deserialize = "UrlKey"))]
    url_key: String,
}

#[derive(Debug, Deserialize)]
struct Items {
    #[serde(rename(deserialize = "UpdateComicItems"))]
    update_comic_items: Vec<Item>,
}

impl Item {
    fn full_url(&self) -> String {
        format!("http://www.manhuaren.com/{}/", self.url_key)
    }
}

def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let client = Client::new();
        let mut params = HashMap::new();
        let page_s = page.to_string();
        params.insert("pageindex", page_s.as_str());
        params.insert("action", "getclasscomics");
        params.insert("pagesize", "21");
        params.insert("categoryid", "0");
        params.insert("tagid", "0");
        params.insert("status", "0");
        params.insert("usergroup", "0");
        params.insert("pay", "1");
        params.insert("areaid", "0");
        params.insert("sort", "2");
        params.insert("iscopyright", "0");
        let items = client
                        .post("http://www.manhuaren.com/manhua-list/dm5.ashx")
                        .form(&params)
                        .send()?
                        .json::<Items>()?;
        let mut comics = vec![];
        for item in &items.update_comic_items {
            comics.push(Comic::new(item.title.clone(), item.full_url()));
        }
        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"http://www.manhuaren.com",
            :target         => &"li > a.chapteritem"
        ]?.attach_to(comic);

        Ok(())
    }

    // fn fetch_pages(&self, chapter: &mut Chapter) -> Result<()> {
    //     Ok(())
    // }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(21, comics.len());

    let mut comic = Comic::from_link(
        "夏目萌记帐 参",
        "http://www.manhuaren.com/manhua-xiamumengjizhang-can/",
    );
    extr.fetch_chapters(&mut comic).unwrap();
    println!("{:?}", comic);
    assert_eq!(1, comic.chapters.len());

    // let chapter1 = &mut comic.chapters[0];
    // chapter1.title = "".to_string();
    // extr.fetch_pages(chapter1).unwrap();
    // assert_eq!("夏目萌记帐 参 第1话", chapter1.title);
    // assert_eq!(21, chapter1.pages.len());
}
