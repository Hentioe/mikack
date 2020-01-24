use super::*;
use base64;
use std::str;

def_regex! [
    CHAPTER_ID_RE   => r#"@type":"ListItem","position":3,"name":"([^"]+)""#,
    IMG_DATA_RE     => r#"<script>\s?var img_data =\s+'([^']+)';</script>"#
];

/// 对 www.manhuadb.com 内容的抓取实现
def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url =
            format!("https://www.manhuadb.com/update_{}.html", page);

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :href_prefix    => &"https://www.manhuadb.com",
            :target         => &r#".comicbook-index  > a"#,
            :text_attr      => &"title"
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"https://www.manhuadb.com",
            :target         => &r#".sort_div > a"#
        ]?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        let chapter_id = match_content![
            :text   => &html,
            :regex  => &*CHAPTER_ID_RE
        ];
        let name = document.dom_text("h1 > a")?;
        chapter.title = format!("{} {}", name, chapter_id);
        let path_prefix = document.dom_attr("div[data-img_pre]", "data-img_pre")?;
        let img_data = match_content![
            :text   => &html,
            :regex  => &*IMG_DATA_RE
        ];
        let decoded_img_data_bytes = &base64::decode(img_data)?[..];
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
        let comic = &mut Comic::from_link("胜负师传说", "https://www.manhuadb.com/manhua/10906");
        extr.fetch_chapters(comic).unwrap();
        assert_eq!(41, comic.chapters.len());
        let chapter1 = &mut comic.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("胜负师传说 VOL01", chapter1.title);
        assert_eq!(194, chapter1.pages.len());
    }
}
