use super::*;
use url::Url;

lazy_static! {
    static ref DEFAULT_DOMAIN_NO: String = "0".to_string();
}

def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = format!("http://www.hhimm.com/comic/{}.html", page);

        itemsgen![
            :entry          => Comic,
            :url            => &url,
            :href_prefix    => &"http://www.hhimm.com",
            :target         => &r#".cComicList > li > a"#
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        itemsgen![
            :entry          => Chapter,
            :url            => &comic.url,
            :href_prefix    => &"http://www.hhimm.com",
            :target         => &".cVolUl > li > a"
        ]?.reversed_attach_to(comic);

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let query_params: HashMap<_, _> = Url::parse(&chapter.url)?
                                .query_pairs()
                                .map(|(name, value)| (name.to_string(), value.to_string()))
                                .collect();
        let domain_no = query_params.get("d").unwrap_or(&*DEFAULT_DOMAIN_NO).clone();
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        let hd_domain_value = document.dom_attr("#hdDomain", "value")?;
        let hd_domain_list = hd_domain_value.split("|").collect::<Vec<_>>();
        let hd_domain = if hd_domain_list.len() > 0 {
            hd_domain_list[0].to_string()
        }else{
            return Err(err_msg("No `hdDomain` found"))
        };
        let s_id = document.dom_attr("#hdVolID", "value")?;
        let s = document.dom_attr("#hdS", "value")?;
        let page_count = document.dom_attr("#hdPageCount", "value")?.parse::<usize>()?;
        chapter.title = document.dom_text("title")?.replace(" - HH漫画 汗汗酷漫", "");

        let fetch = Box::new(move |current_page| {
            let page_url = format!("http://www.hhimm.com/cool{s_id}/{i}.html?s={s}&d={domain_no}",
                s_id=s_id, i=current_page, s=s, domain_no=domain_no
            );
            let html = get(&page_url)?.text()?;
            let document = parse_document(&html);
            let img_name_attr = document.dom_attr("#iBodyQ img", "name")?;
            let runtime = include_str!("../../runtime/hhimm.js");
            let wrap_code = wrap_code!(runtime, format!("
                var location = {{ hostname: '{}' }};
                unsuan('{}')
            ", "www.hhimm.com", img_name_attr), :end);
            let path = eval_as::<String>(&wrap_code)?;
            let address = format!("{}{}", hd_domain, path);

            Ok(vec![Page::new(current_page - 1, address)])
        });

        Ok(ChapterPages::new(chapter, page_count as i32, vec![], fetch))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(30, comics.len());

    let mut comic = Comic::new("妖精的尾巴", "http://www.hhimm.com/manhua/2779.html");
    extr.fetch_chapters(&mut comic).unwrap();
    assert_eq!(597, comic.chapters.len());

    let chapter1 = &mut Chapter::from_link("", "http://www.hhimm.com/cool285664/1.html?s=4");
    extr.fetch_pages_unsafe(chapter1).unwrap();
    assert_eq!("妖精的尾巴 543集", chapter1.title);
    assert_eq!(22, chapter1.pages.len());
}
