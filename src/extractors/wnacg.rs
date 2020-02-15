use super::*;
use std::cell::RefCell;
use std::vec::Vec;

def_regex2![
    ID      => r#"https?://www\.wnacg\.org/photos-index-(page-\d+-)?aid-(\d+)\.html"#,
    COUNT   => r#"頁數：(\d+)P"#,
    NEXT    => r#"imagePreload\.src = '([^']+)';"#
];

/// 对 www.wnacg.org 内容的抓取实现
def_extractor! {
	state	=> [usable: true, pageable: true, searchable: true],
	tags	=> [Chinese, NSFW],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://www.wnacg.org/albums-index-page-{}.html", page);

        let mut comics = itemsgen2!(
            url             = &url,
            parent_dom      = ".cc > .gallary_item",
            cover_dom       = "a > img",
            link_dom        = ".pic_box > a",
            link_prefix     = "https://www.wnacg.org",
            link_text_attr  = "title"
        )?;
        comics.iter_mut().for_each(|comic: &mut Comic| {
            comic.cover = comic.cover.replace("//", "https://");
        });

        Ok(comics)
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.wnacg.org/albums-index-page-1-sname-{}.html", keywords);

        let mut comics = itemsgen2!(
            url             = &url,
            parent_dom      = ".cc > .gallary_item",
            cover_dom       = "a > img",
            link_dom        = ".pic_box > a",
            link_prefix     = "https://www.wnacg.org",
            link_text_attr  = "title"
        )?;
        comics.iter_mut().for_each(|comic: &mut Comic| {
            comic.cover = comic.cover.replace("//", "https://");
        });

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.chapters.push(Chapter::from(&*comic));

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let id = match_content2!(&chapter.url, &*ID_RE, group = 2)?;
        let make_page_url = move |page: usize| -> String {
            format!("https://www.wnacg.org/photos-index-page-{page}-aid-{id}.html", page = page, id = id)
        };
        chapter.url = make_page_url(1);

        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);

        chapter.set_title(document.dom_attr(".uwthumb > img", "alt")?);
        let total = match_content2!(
            &document.dom_text(".uwconn > label:nth-child(2)")?,
            &*COUNT_RE
        )?.parse::<i32>()?;

        let preview_documents = RefCell::new(Vec::<Html>::new());
        let fetch_preview_document = move |current_page: usize| -> Result<Html> {
            let page_num = (current_page as f64 / 12.0).ceil() as usize;
            if preview_documents.borrow().len() < page_num { // 载入页面
                let preview_html = get(&make_page_url(page_num))?.text()?;
                let preview_docuement = parse_document(&preview_html);
                {
                    preview_documents.try_borrow_mut()?.push(preview_docuement.clone());
                }

                Ok(preview_docuement)
            } else { // 返回网页文档
                let preview_docuement = preview_documents.borrow()[page_num - 1].clone();

                Ok(preview_docuement)
            }
        };
        let fetch = Box::new(move |current_page: usize| {
            let preview_document = fetch_preview_document(current_page)?;
            let element_num = if current_page < 12 {
                current_page
            } else {
                let rem = current_page % 12;
                if rem == 0 { 12 } else { rem }
            };
            let page_path = preview_document.dom_attr(
                &format!(".cc > li:nth-child({}) > .pic_box > a", element_num),
                "href"
            )?;
            let page_html = get(&format!("https://www.wnacg.org{}", page_path))?.text()?;
            let page_document = parse_document(&page_html);
            let mut pages = vec![];
            let first_address = format!(
                "https:{}",
                page_document.dom_attr("#picarea", "src")?
            );
            pages.push(Page::new(current_page - 1, first_address));
            if current_page < total as usize {
                let next_address = format!(
                    "https:{}",
                    match_content2!(&page_html, &*NEXT_RE)?
                );
                pages.push(Page::new(current_page, next_address));
            }

            Ok(pages)
        });

        Ok(ChapterPages::new(chapter, total, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(12, comics.len());
        let mut comic1 = Comic::new(
            "[雛咲葉] ワルイヤツ [COMlC 快楽天ビースト 2017年2月号][不想记名汉化][无修正]",
            "https://www.wnacg.org/photos-index-aid-89569.html",
        );
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(1, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!(
            "[雛咲葉] ワルイヤツ [COMlC 快楽天ビースト 2017年2月号][不想记名汉化][无修正]",
            chapter1.title
        );
        assert_eq!(27, chapter1.pages.len());
        let comics = extr.search("ワルイヤツ").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
