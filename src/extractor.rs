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
use scraper::{element_ref::ElementRef, Html, Selector};

fn parse_selector(selector: &str) -> Result<Selector> {
    Ok(Selector::parse(selector)
        .map_err(|_e| err_msg(format!("The selector '{}' parsing failed", selector)))?)
}

fn parse_document(html: &str) -> Html {
    Html::parse_document(html)
}

fn simple_fetch_index<T: FromLink>(
    url: &str,
    selector: &str,
    parse_elem: &dyn Fn(ElementRef) -> Result<T>,
) -> Result<Vec<T>> {
    let html = get(url)?.text()?;
    let document = parse_document(&html);
    let mut list = Vec::new();

    for element in document.select(&parse_selector(selector)?) {
        list.push(parse_elem(element)?);
    }

    Ok(list)
}

fn simple_parse_link(element: ElementRef, selector: &str) -> Result<(String, String)> {
    let link_elem = element
        .select(&parse_selector(selector)?)
        .next()
        .ok_or(err_msg("No link found"))?;
    let title = link_elem.text().next().ok_or(err_msg("No title found"))?;
    let url = link_elem
        .value()
        .attr("href")
        .ok_or(err_msg("No href found"))?;

    Ok((title.to_string(), url.to_string()))
}

macro_rules! def_exctractor {
    ($name:ident => { $($tt:tt)* }) => {
        pub struct $name;
        impl Extractor for $name {
            $($tt)*
        }
    };
}

macro_rules! urlgen {
    (:first => $first:expr, :next => $next:expr, :page => $page:expr, :seed => $seed:expr) => {
        if $page > 0 {
            format!($next, $page + $seed)
        } else {
            format!($first)
        }
    };
}

macro_rules! itemsgen {
    (:entry => $entry:ident, :url => $url:expr, :selector => $selector:expr, :find => $find:expr) => {
        simple_fetch_index($url, $selector, &|element: ElementRef| {
            let (title, url) = simple_parse_link(element, $find)?;
            Ok($entry::from_link(title, url))
        })
    };
}

def_exctractor!(Dmzj => { 
    fn index(&self, page: u32) -> Result<Vec<Comic>> {
        let url = urlgen![
            :first  => "https://manhua.dmzj.com/rank/",
            :next   => "https://manhua.dmzj.com/rank/total-block-{}.shtml",
            :page   => page,
            :seed   => 1
        ];

        itemsgen![
            :entry      => Comic,
            :url        => &url,
            :selector   => ".middleright-right > .middlerighter",
            :find       => ".title > a"
        ]
    }

    fn fetch_chapters(&self, comic: &mut Comic) -> Result<()> {
        for (i, chapter) in itemsgen![
            :entry      => Chapter,
            :url        => &comic.url,
            :selector   => ".cartoon_online_border > ul > li",
            :find       => "a"
        ]?.iter_mut().enumerate(){
            chapter.which = (i as u32) + 1;
            chapter.title = format!("{} {}", &comic.title, &chapter.title);
            comic.push_chapter(chapter.clone());
        };

        Ok(())
    }
});
