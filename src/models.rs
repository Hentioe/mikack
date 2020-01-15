use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Page {
    pub n: usize,
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct Chapter {
    pub title: String,
    pub url: String,
    pub which: u32,
    pub pages: Vec<Page>,
    pub page_headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Comic {
    pub title: String,
    pub url: String,
    pub chapters: Vec<Chapter>,
}

impl Page {
    pub fn new<S: Into<String>>(n: usize, address: S) -> Self {
        Self {
            n,
            address: address.into(),
        }
    }
}

impl Chapter {
    pub fn new<S: Into<String>>(title: S, url: S, which: u32) -> Self {
        let mut page_headers = HashMap::new();
        let url = url.into();
        page_headers.insert(String::from("Referer"), url.clone());
        Self {
            title: title.into(),
            url: url.clone(),
            which,
            pages: vec![],
            page_headers,
        }
    }

    pub fn push_page(&mut self, page: Page) {
        self.pages.push(page);
    }

    pub fn push_page_header<S: Into<String>>(&mut self, key: S, value: S) {
        self.page_headers.insert(key.into(), value.into());
    }
}

impl Comic {
    pub fn new<S: Into<String>>(title: S, url: S) -> Self {
        Self {
            title: title.into(),
            url: url.into(),
            chapters: vec![],
        }
    }

    pub fn push_chapter(&mut self, chapter: Chapter) {
        self.chapters.push(chapter);
    }
}

pub trait FromLink {
    fn from_link<S: Into<String>>(text: S, href: S) -> Self;
}

impl FromLink for Comic {
    fn from_link<S: Into<String>>(text: S, href: S) -> Self {
        Self::new(text, href)
    }
}

impl FromLink for Chapter {
    fn from_link<S: Into<String>>(text: S, href: S) -> Self {
        Self::new(text, href, 0)
    }
}
