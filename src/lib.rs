#[macro_use]
extern crate failure;

pub mod error;
pub mod extractor;
pub mod models;

#[cfg(test)]
mod tests {
    use crate::models::{Chapter, Comic, Page};

    #[test]
    fn test_init_models() {
        let p1 = Page::new(1, "https//libmanga.com/commins/1/1.jpg");
        let p2 = Page::new(2, "https//libmanga.com/commins/1/2.jpg");

        let mut chapter = Chapter::new("一圈超人 第一话", 1);
        chapter.push_page(p1);
        chapter.push_page(p2);

        let mut comic = Comic::new("一拳超人", "https//libmanga.com/commins/1");
        comic.push_chapter(chapter);
    }

    use crate::extractor::*;

    #[test]
    fn test_extractor() {
        let comics = Dmzj {}.index(0).unwrap();
        assert_eq!(20, comics.len());
    }
}
