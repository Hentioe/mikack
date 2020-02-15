use super::*;
use serde::Deserialize;
use serde_json;

#[derive(Debug, Deserialize)]
struct SearchJson {
    search_data: Vec<SearchComicItem>,
}

#[derive(Debug, Deserialize)]
struct SearchComicItem {
    comic_name: String,
    comic_cover: String,
    comic_url_raw: String,
}

impl SearchComicItem {
    fn comic_url_full(&self) -> String {
        self.comic_url_raw.replace("//", "https://").to_string()
    }
}

impl From<SearchComicItem> for Comic {
    fn from(c: SearchComicItem) -> Self {
        Self::from_index(&c.comic_name, &c.comic_url_full(), &c.comic_cover)
    }
}

def_regex2![
    CTYPTO  => r#"<script type="text/javascript">([\s\S]+)var res_type"#,
    DATA    => r#"var g_search_data =\s?(\[.+\]);"#
];

def_extractor! {
	state	=> [usable: true, pageable: true, searchable: true],
	tags	=> [Chinese],

    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("https://manhua.dmzj.com/update_{}.shtml", page);

        itemsgen2!(
            url             = &url,
            parent_dom      = ".newpic_content .boxdiv1",
            cover_dom       = "a > img",
            link_dom        = "a.pictextst",
            link_prefix     = "https://manhua.dmzj.com/"
        )
    }

    fn search(&self, keywords: &str) -> Result<Vec<Comic>> {
        let url = format!("https://sacg.dmzj.com/comicsum/search.php?s={}", keywords);
        let html = get(&url)?.text()?;
        let search_data = match_content2!(&html, &*DATA_RE)?;
        let search_data_json = format!("{{ \"search_data\": {} }}", search_data);
        let json: SearchJson = serde_json::from_str(&search_data_json)?;

        let mut comics = vec![];
        for c in json.search_data {
            comics.push(Comic::from(c));
        }

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen2!(
            url            = &comic.url,
            target_dom     = ".cartoon_online_border > ul > li > a",
            link_prefix    = "http://manhua.dmzj.com"
        )?.attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let code = match_content2!(&html, &*CTYPTO_RE)?;
        let wrap_code = format!("{}\n{}", &code, "
            var obj = {
                title: `${g_comic_name} ${g_chapter_name}`,
                pages: eval(pages)
            };
            obj
        ");
        let obj = eval_as_obj(&wrap_code)?;
        chapter.title = obj.get_as_string("title")?.clone();
        let mut addresses = vec![];
        for path in obj.get_as_array("pages")? {
            let address = format!("https://images.dmzj.com/{}", path.as_string()?);
            addresses.push(address);
        }

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    if extr.is_usable() {
        let comics = extr.index(1).unwrap();
        assert_eq!(40, comics.len());
        let mut comic1 = Comic::from_link(
            "灌篮高手全国大赛篇(全彩版本)",
            "https://manhua.dmzj.com/lanqiufeirenquancai",
        );
        extr.fetch_chapters(&mut comic1).unwrap();
        assert_eq!(80, comic1.chapters.len());
        let chapter1 = &mut comic1.chapters[0];
        extr.fetch_pages_unsafe(chapter1).unwrap();
        assert_eq!("灌篮高手全国大赛篇(全彩版本) 第01话", chapter1.title);
        assert_eq!(21, chapter1.pages.len());
        let comics = extr.search("灌篮高手全国大赛篇(全彩版本)").unwrap();
        assert!(comics.len() > 0);
        assert_eq!(comics[0].title, comic1.title);
        assert_eq!(comics[0].url, comic1.url);
    }
}
