use super::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Cover {
    url: String,
}

#[derive(Debug, Deserialize)]
struct Item {
    title: String,
    url: String,
    cover: Cover,
}

#[derive(Debug, Deserialize)]
struct List {
    items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Album {
    list: List,
}

#[derive(Debug, Deserialize)]
struct Data {
    album: Album,
}

#[derive(Debug, Deserialize)]
struct Json {
    data: Data,
}

def_exctractor! {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let mut url = String::from(r#"https://api.luscious.net/graphql/nobatch/?operationName=AlbumList&query=+query+AlbumList($input:+AlbumListInput!)+{+album+{+list(input:+$input)+{+info+{+...FacetCollectionInfo+}+items+{+...AlbumMinimal+}+}+}+}+fragment+FacetCollectionInfo+on+FacetCollectionInfo+{+page+has_next_page+has_previous_page+total_items+total_pages+items_per_page+url_complete+}+fragment+AlbumMinimal+on+Album+{+__typename+id+title+labels+description+created+modified+like_status+status+number_of_favorites+number_of_dislikes+number_of_pictures+number_of_animated_pictures+number_of_duplicates+slug+is_manga+url+download_url+permissions+created_by+{+id+url+name+display_name+user_title+avatar+{+url+size+}+}+cover+{+width+height+size+url+}+content+{+id+title+url+}+language+{+id+title+url+}+tags+{+id+category+text+url+count+}+genres+{+id+title+slug+url+}+audiences+{+id+title+url+}+}+&variables={"input":{"display":"date_trending","filters":[{"name":"album_type","value":"manga"}],"page":"#);
        url.push_str(&page.to_string());
        url.push_str("}}");
        let json = get(&url)?.json::<Json>()?;
        let mut comics = vec![];
        for item in json.data.album.list.items {
            comics.push(Comic::from_index(item.title, format!("https://www.luscious.net{}", item.url), item.cover.url))
        }

        Ok(comics)
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        comic.push_chapter(Chapter::from_link(comic.title.clone(), comic.url.clone()));
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
                addr.replace("315x0", "1024x0").to_string()
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

    let chapter1 = &mut Chapter::from_link("", "https://www.luscious.net/albums/teitoku-wa-semai-toko-suki-kantai-collection-kanco_363520/");
    extr.fetch_pages(chapter1).unwrap();
    assert_eq!(
        "Teitoku wa Semai Toko Suki (Kantai Collection -KanColle-) [English]",
        chapter1.title
    );
    assert_eq!(26, chapter1.pages.len());
}
