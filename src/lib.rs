pub mod models;

#[cfg(test)]
mod tests {
    use crate::models::{Chapter, Comic, Page};

    #[test]
    fn test_init_models() {
        let p1 = Page::new(1, "https//libmanga.com/commins/1/1.jpg");
        let p2 = Page::new(2, "https//libmanga.com/commins/1/2.jpg");

        let mut chapter = Chapter::new("一圈超人 第一话", 1);
        chapter.push_page(p1);
        chapter.push_page(p2);

        let mut comic = Comic::new("一拳超人");
        comic.push_chapter(chapter);
    }

    #[test]
    fn test_scrap_data() {
        let resp = reqwest::blocking::get("https://www.bing.com").unwrap();
        assert_eq!(reqwest::StatusCode::OK, resp.status());
        let html = resp.text().unwrap();
        let document = scraper::Html::parse_document(&html);
        let selector = scraper::Selector::parse("title").unwrap();
        let title = document
            .select(&selector)
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()[0];

        assert_eq!(true, title.contains("Bing"))
    }
}
