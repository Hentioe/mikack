use super::*;
use scraper::Html;

pub trait HtmlExt {
    fn dom_texts(&self, selector: &str) -> Result<Vec<String>>;
    fn dom_text(&self, selector: &str) -> Result<String> {
        let texts = self.dom_texts(selector)?;
        if texts.len() == 0 {
            Err(err_msg(format!("DOM node not found: {}", selector)))
        } else {
            Ok(texts[0].clone())
        }
    }
    fn dom_attrs(&self, selector: &str, attr: &str) -> Result<Vec<String>>;
    fn dom_attr(&self, selector: &str, attr: &str) -> Result<String> {
        let attrs = self.dom_attrs(selector, attr)?;
        if attrs.len() == 0 {
            Err(err_msg(format!("DOM node not found: {}", selector)))
        } else {
            Ok(attrs[0].clone())
        }
    }
    fn dom_count(&self, selector: &str) -> Result<usize>;
}

impl HtmlExt for Html {
    fn dom_texts(&self, selector: &str) -> Result<Vec<String>> {
        let mut texts = vec![];

        for element in self.select(&parse_selector(selector)?) {
            let text = element
                .text()
                .next()
                .ok_or(err_msg(format!("Text not found in `{}`", selector)))?
                .trim()
                .to_string();
            texts.push(text);
        }

        Ok(texts)
    }

    fn dom_attrs(&self, selector: &str, attr: &str) -> Result<Vec<String>> {
        let mut attrs = vec![];

        for element in self.select(&parse_selector(selector)?) {
            let attr_s = element.value().attr(&attr).ok_or(err_msg(format!(
                "Attribute `{}` not found in `{}`",
                attr, selector
            )))?;
            attrs.push(attr_s.trim().to_string());
        }

        Ok(attrs)
    }

    fn dom_count(&self, selector: &str) -> Result<usize> {
        Ok(self
            .select(&parse_selector(selector)?)
            .collect::<Vec<_>>()
            .len())
    }
}
