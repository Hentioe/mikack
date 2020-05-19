use super::*;
use serde::Deserialize;

def_regex2![
    SCRIPT    => r#"window\.(__NUXT__=\S?\(.+\);)</script>"#,
];

#[derive(Debug, Deserialize)]
struct IndexJson {
    data: IndexData,
}

#[derive(Debug, Deserialize)]
struct IndexData {
    topics: Vec<Topic>,
}

#[derive(Debug, Deserialize)]
struct SearchJson {
    data: SearchData,
}

#[derive(Debug, Deserialize)]
struct SearchData {
    hit: Vec<Topic>,
}

#[derive(Debug, Deserialize)]
struct Topic {
    id: i64,
    title: String,
    vertical_image_url: String,
}

impl Topic {
    fn url(&self) -> String {
        format!("https://www.kuaikanmanhua.com/web/topic/{}/", self.id)
    }
}

// 对 www.kuaikanmanhua.com 内容的抓取实现
def_extractor! {
    status	=> [
        usable: true, searchable: true, pageable: true, https: true,
        favicon: "https://www.kuaikanmanhua.com/favicon.ico"
    ],
    tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!(
            "https://www.kuaikanmanhua.com/v1/search/by_tag?since={since}&count=48&f=3&tag=0&query_category=%7B%22update_status%22:1%7D",
            since = (page - 1) * 48
        );

        let json = get(&url)?.json::<IndexJson>()?;
        let mut comics = vec![];
        for topic in json.data.topics {
            comics.push(Comic::from_index(&topic.title, &topic.url(), &topic.vertical_image_url));
        }

        Ok(comics)
    }

    fn search(&self, keyworkds: &str) -> Result<Vec<Comic>> {
        let url = format!("https://www.kuaikanmanhua.com/v1/search/topic?q={}&f=3&size=18", keyworkds);

        let json = get(&url)?.json::<SearchJson>()?;
        let mut comics = vec![];
        for topic in json.data.hit {
            comics.push(Comic::from_index(&topic.title, &topic.url(), &topic.vertical_image_url));
        }

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url             = &comic.url,
            target_dom      = r#".TopicItem > .title > a[href^="/"]"#,
            target_text_dom = "span",
            link_prefix     = "https://www.kuaikanmanhua.com"
        )?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(
            format!("{} {}",
                document.dom_text(".titleBox > h3.title > a:nth-child(3)")?,
                document.dom_text("title")?.split("|").next().ok_or(err_msg("No title found"))?
            )
        );
        let script_code = match_content2!(&html, &*SCRIPT_RE)?;
        let wrap_code = format!("
            {script}
            __NUXT__.data[0].comicInfo.comicImages.map((c) => c.url)
        ", script = script_code);

        let value = eval_value(&wrap_code)?;
        let mut addresses = vec![];
        for addr in value.as_array()? {
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
        assert_eq!(47, comics.len());
        let comic1 =
            &mut Comic::from_link("整容游戏", "https://www.kuaikanmanhua.com/web/topic/544/");
        extr.fetch_chapters(comic1).unwrap();
        assert_eq!(18, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("整容游戏 第1话 整容游戏APP", chapter1.title);
        assert_eq!(146, chapter1.pages.len());

        let comics = extr.search("整容游戏").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
