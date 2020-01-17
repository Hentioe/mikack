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

def_regex! {
    DECRYPT_RE => r#"(eval\(.+\))[\s\S]*</script>"#
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
        let list_1 = itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"http://www.manhuaren.com",
            :target         => &"ul.detail-list-1 > li > a.chapteritem"
        ]?;
        let list_2 = itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"http://www.manhuaren.com",
            :target         => &"ul.detail-list-2 > li > a.chapteritem",
            :sub_dom_text   => &".detail-list-2-info-title"
        ]?;

        if list_1.len() > 0 {
            list_1.reversed_attach_to(comic);
        }
        if list_2.len() > 0 {
            list_2.attach_to(comic);
        }

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        let title = document.dom_text("p.view-fix-top-bar-title")?;
        chapter.title =  Chapter::title(&title[0..(title.len() - 1)]);

        let decrypt_code = match_content![
            :text   =>  &html,
            :regex  => &*DECRYPT_RE
        ];
        let wrap_code = wrap_code!(&decrypt_code, format!("
            eval({})
        ", "newImgs"), :end);

        let mut addresses = vec![];
        for img in eval_value(&wrap_code)?.as_array()? {
            addresses.push(img.as_string()?.clone());
        }

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(21, comics.len());

    let mut comic = Comic::from_link(
        "风云全集",
        "https://www.manhuaren.com/manhua-fengyunquanji/",
    );
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(670, comic.chapters.len());

    let chapter1 = &mut comic.chapters[0];
    extr.fetch_pages(chapter1).unwrap();
    assert_eq!("风云全集外传：第1话", chapter1.title);
    assert_eq!(53, chapter1.pages.len());
}
