use super::*;
use reqwest::blocking::multipart::Form;
use reqwest::blocking::multipart::Part;
use reqwest::blocking::Client;
use std::str;

def_regex2![
    PACKED  => r#"packed="([^"]+)""#
];

/// 对 www.ipufei.com 内容的抓取实现
def_extractor! {[usable: true, pageable: true, searchable: true],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = urlgen2!(page,
            first   = "http://www.ipufei.com/shaonianrexue/index.html",
            next    = "http://www.ipufei.com/shaonianrexue/index_{}.html"
        );

        itemsgen2!(
            url             = &url,
            encoding        = GBK,
            parent_dom      = ".dmList > ul > li",
            cover_dom       = "p.cover > a > img",
            cover_attr      = "_src",
            link_dom        = "dt > a",
            link_prefix     = "http://www.ipufei.com"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = "http://www.ipufei.com/e/search/index.php";
        let bytes = &encode_text(keywords, GBK)?[..];
        let keyboard_part = Part::bytes(bytes.to_vec());
        let form = Form::new()
            .text("orderby", "1")
            .text("myorder", "1")
            .text("tbname", "mh")
            .text("tempid", "3")
            .text("show", "title,player,playadmin,bieming,pinyin")
            .part("keyboard", keyboard_part);
        let client = Client::new();
        let html = client
            .post(url)
            .multipart(form)
            .send()?
            .text()?;

        itemsgen2!(
            html            = &html,
            encoding        = GBK,
            parent_dom      = ".dmList > ul > li",
            cover_dom       = "p.cover > a > img",
            cover_attr      = "_src",
            link_dom        = "dt > a",
            link_prefix     = "http://www.ipufei.com"
        )
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            encoding        = GBK,
            target_dom      = ".plist > ul > li > a",
            link_prefix     = "http://www.ipufei.com"
        )?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.decode_text(GBK)?;
        let document = parse_document(&html);
        chapter.set_title(document.dom_text("h1")?);
        let packed = match_content2!(&html, &*PACKED_RE)?;
        let wrap_code = wrap_code!(include_str!("../../runtime/ipufei.js"), format!("
            var data = decode('{packed}');
            data
        ", packed = packed), :end);
        let mut addresses = vec![];
        for addr in eval_value(&wrap_code)?.as_array()? {
            addresses.push(addr.as_string()?.clone());
        }

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(25, comics.len());
        let mut comic1 = Comic::new("俺物语", "http://www.ipufei.com/manhua/600/index.html");
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(65, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("俺物语 第1话", chapter1.title);
        assert_eq!(101, chapter1.pages.len());
        let comics = extr.search("俺物语").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
