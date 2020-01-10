#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;

pub mod error;
pub mod extractor;
pub mod jsrt;
pub mod models;

#[cfg(test)]
mod tests {
    use crate::extractor::*;
    use crate::models::*;

    #[test]
    fn test_extractor() {
        let dmjz = Dmzj {};
        let comics = dmjz.index(0).unwrap();
        assert_eq!(20, comics.len());

        let mut comic = Comic::from_link("极主夫道", "http://manhua.dmzj.com/jizhufudao/");
        dmjz.fetch_chapters(&mut comic).unwrap();
        assert_eq!(47, comic.chapters.len());

        let mut chapter = Chapter::from_link(
            "极主夫道 第01话",
            "http://manhua.dmzj.com/jizhufudao/73104.shtml",
        );
        dmjz.fetch_pages(&mut chapter).unwrap();
    }
}
