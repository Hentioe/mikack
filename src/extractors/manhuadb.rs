use super::*;
use base64;
use std::str;

def_regex2! [
    CHAPTER_ID   => r#"@type":"ListItem","position":3,"name":"([^"]+)""#,
    IMG_DATA     => r#"<script>\s?var img_data =\s+'([^']+)';</script>"#
];

/// 对 www.manhuadb.com 内容的抓取实现
def_extractor! {
	state	=> [usable: true, pageable: true, searchable: true],
	tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.manhuadb.com/update_{}.html", page);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".comicbook-index",
            cover_dom       = ".img-fluid",
            link_dom        = ".one-line > a",
            link_prefix     = "https://www.manhuadb.com"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.manhuadb.com/search?q={}", keywords);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".comicbook-index",
            cover_dom       = ".img-fluid",
            link_dom        = ".one-line > a",
            link_prefix     = "https://www.manhuadb.com"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = ".sort_div > a",
            link_prefix     = "https://www.manhuadb.com"
        )?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        let chapter_id = match_content2!(&html, &*CHAPTER_ID_RE)?;
        let name = document.dom_text("h1 > a")?;
        chapter.title = format!("{} {}", name, chapter_id);
        let path_prefix = document.dom_attr("div[data-img_pre]", "data-img_pre")?;
        let img_data = match_content2!(&html, &*IMG_DATA_RE)?;
        let decoded_img_data_bytes = &base64::decode(&img_data)?[..];
        let decoded_img_data = str::from_utf8(decoded_img_data_bytes)?;
        let wrap_code = format!("
            var imgs = eval('{decoded_img_data}').map(item => item.img);
            imgs
        ", decoded_img_data = decoded_img_data);
        let mut addresses = vec![];
        for img_file in eval_value(&wrap_code)?.as_array()? {
            addresses.push(format!("https://www.manhuadb.com{}{}", path_prefix, img_file.as_string()?));
        }

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(36, comics.len());
        let comic1 = &mut Comic::from_link("胜负师传说", "https://www.manhuadb.com/manhua/10906");
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(41, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("胜负师传说 VOL01", chapter1.title);
        assert_eq!(194, chapter1.pages.len());
        let comics = extr.search("胜负师传说").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
