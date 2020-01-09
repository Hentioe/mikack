use crate::error::*;
use crate::models::*;

pub trait Extractor {
    fn index(&self, page: u32) -> Result<Vec<Comic>>;
    fn fetch_chapters(&self, _comic: &mut Comic) -> Result<()> {
        Ok(())
    }
    fn fetch_pages(&self, _chapter: &mut Chapter) -> Result<()> {
        Ok(())
    }
}

use reqwest::blocking::get;
use scraper::{Html, Selector};

fn parse_selector(selector: &str) -> Result<Selector> {
    Ok(Selector::parse(selector)
        .map_err(|_e| err_msg(format!("The selector '{}' parsing failed", selector)))?)
}

fn parse_document(html: &str) -> Html {
    Html::parse_document(html)
}

const NO_LINK_FOUND: &'static str = "No link found";
const NO_TITLE_FOUND: &'static str = "No title found";

pub struct Dmzj;

impl Extractor for Dmzj {
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = if page > 0 {
            format!(
                "https://manhua.dmzj.com/rank/total-block-{}.shtml",
                page + 1
            )
        } else {
            format!("https://manhua.dmzj.com/rank/")
        };

        let html = get(&url)?.text()?;
        let document = parse_document(&html);
        let mut comics = Vec::new();

        for element in document.select(&parse_selector(".middleright-right > .middlerighter")?) {
            let link_elem = element
                .select(&parse_selector(".title > a")?)
                .next()
                .ok_or(err_msg(NO_LINK_FOUND))?;
            let title = link_elem.text().next().ok_or(err_msg(NO_TITLE_FOUND))?;
            let url = link_elem
                .value()
                .attr("href")
                .ok_or(err_msg(NO_LINK_FOUND))?;

            comics.push(Comic::new(title, url));
        }

        Ok(comics)
    }
}
