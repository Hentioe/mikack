use super::*;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct Cover {
    url: String,
}

#[derive(Debug, Deserialize)]
struct ComicItem {
    title: String,
    url: String,
    cover: Cover,
}

impl From<&ComicItem> for Comic {
    fn from(c: &ComicItem) -> Self {
        Self::from_index(
            &c.title,
            &format!("https://www.luscious.net{}", &c.url),
            &c.cover.url,
        )
    }
}

/// 对 www.luscious.net 内容的抓取实现
def_extractor! {[usable: true, pageable: true, searchable: false],
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let mut url = String::from(r#"https://api.luscious.net/graphql/nobatch/?operationName=AlbumList&query=+query+AlbumList($input:+AlbumListInput!)+{+album+{+list(input:+$input)+{+info+{+...FacetCollectionInfo+}+items+{+...AlbumMinimal+}+}+}+}+fragment+FacetCollectionInfo+on+FacetCollectionInfo+{+page+has_next_page+has_previous_page+total_items+total_pages+items_per_page+url_complete+}+fragment+AlbumMinimal+on+Album+{+__typename+id+title+labels+description+created+modified+like_status+status+number_of_favorites+number_of_dislikes+number_of_pictures+number_of_animated_pictures+number_of_duplicates+slug+is_manga+url+download_url+permissions+created_by+{+id+url+name+display_name+user_title+avatar+{+url+size+}+}+cover+{+width+height+size+url+}+content+{+id+title+url+}+language+{+id+title+url+}+tags+{+id+category+text+url+count+}+genres+{+id+title+slug+url+}+audiences+{+id+title+url+}+}+&variables={"input":{"display":"date_trending","filters":[{"name":"album_type","value":"manga"}],"page":"#);
        url.push_str(&page.to_string());
        url.push_str("}}");
        let json_v = get(&url)?.json::<Value>()?;
        let list = json_v["data"]["album"]["list"]["items"].clone();
        let comics = serde_json::from_value::<Vec<ComicItem>>(list)?
            .iter()
            .map(|c: &ComicItem| {
                Comic::from(c)
            })
            .collect::<Vec<_>>();

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.push_chapter(Chapter::from(&*comic));

        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        let html = get(&chapter.url)?.text()?;
        let document = parse_document(&html);
        chapter.set_title(document.dom_text("h3.o-h3.o-row-gut-half > a")?);
        let addresses = document
            .dom_attrs(".ReactVirtualized__Grid__innerScrollContainer .picture-card-outer > img", "src")?
            .iter()
            .map(|addr| {
                addr
                    .replace("w315", "w1024")
                    .replace("315x0", "1024x0")
                    .to_string()
            })
            .collect::<Vec<_>>();

        Ok(ChapterPages::full(chapter, addresses))
    }
}

#[test]
fn test_extr() {
    let extr = new_extr();
    let comics = extr.index(1).unwrap();
    assert_eq!(30, comics.len());

    let comic1 = Comic::from_link(
        "Teitoku wa Semai Toko Suki (Kantai Collection -KanColle-) [English]", 
        "https://www.luscious.net/albums/teitoku-wa-semai-toko-suki-kantai-collection-kanco_363520/"
    );
    let chapter1 = &mut Chapter::from(&comic1);
    extr.fetch_pages_unsafe(chapter1).unwrap();
    assert_eq!(
        "Teitoku wa Semai Toko Suki (Kantai Collection -KanColle-) [English]",
        chapter1.title
    );
    assert_eq!(26, chapter1.pages.len());
    // let comics = extr.search("Teitoku wa Semai Toko Suki").unwrap();
    // assert!(comics.len() > 0);
    // assert_eq!(comics[0].title, comic1.title);
    // assert_eq!(comics[0].url, comic1.url);
}
