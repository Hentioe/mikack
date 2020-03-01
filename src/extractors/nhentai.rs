use super::*;

/// 对 nhentai.net 内容的抓取实现
def_extractor! {
    status	=> [
		usable: true, pageable: true, searchable: true, https: true,
		favicon: "https://nhentai.net/favicon.ico"
	],
    tags	=> [English, Japanese, Chinese, NSFW],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://nhentai.net/?page={}", page);

        let mut comics = itemsgen2!(
            url             = &url,
            parent_dom      = ".index-container > .gallery",
            cover_dom       = "a > img",
            cover_attrs     = &["data-src", "src"],
            link_dom        = "a.cover",
            link_prefix     = "https://nhentai.net",
            link_text_dom   = ".caption"
        )?;
        comics.iter_mut().for_each(|comic: &mut Comic| {
            comic.cover = comic
                .cover
                .replace("//t.nhentai.net", "https://t.nhentai.net")
                .to_string();
        });

        Ok(comics)
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://nhentai.net/search/?q={}", keywords);

        let mut comics = itemsgen2!(
            url             = &url,
            parent_dom      = ".index-container > .gallery",
            cover_dom       = "a > img",
            cover_attrs     = &["data-src", "src"],
            link_dom        = "a.cover",
            link_prefix     = "https://nhentai.net",
            link_text_dom   = ".caption"
        )?;
        comics.iter_mut().for_each(|comic: &mut Comic| {
            comic.cover = comic
                .cover
                .replace("//t.nhentai.net", "https://t.nhentai.net")
                .to_string();
        });

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.chapters.push(Chapter::from_link(&comic.title, &comic.url));

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(document.dom_text("#info > h1")?);
        let addresses = document
            .dom_attrs(".thumb-container > a > img", "data-src")?
            .iter()
            .map(|addr| {
                addr
                    .replace("t.jpg", ".jpg")
                    .replace("t.nhentai.net", "i.nhentai.net")
                    .to_string()
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
        assert_eq!(25, comics.len());
        let mut comic1 = Comic::new(
            "(Shuuki Reitaisai) [Haitokukan (Haitokukan)] Touhou Deisuikan 1 Usami Renko (Touhou Project) [Chinese]",
            "https://nhentai.net/g/300773/"
        );
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(1, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!(
            "(Shuuki Reitaisai) [Haitokukan (Haitokukan)] Touhou Deisuikan 1 Usami Renko (Touhou Project) [Chinese]",
            chapter1.title
        );
        assert_eq!(22, chapter1.pages.len());
        let comics = extr.search("Touhou Deisuikan 1 Usami Renko").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
