use crate::error::*;
use scraper::{Html, Selector};

pub fn parse_selector(selectors: &str) -> Result<Selector> {
    Ok(Selector::parse(selectors)
        .map_err(|_e| err_msg(format!("The selectors `{}` parsing failed", selectors)))?)
}

pub fn parse_document(html: &str) -> Html {
    Html::parse_document(html)
}

pub mod document_ext;
pub mod grouped_items;
