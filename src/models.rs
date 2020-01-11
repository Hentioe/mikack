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
    current_p: u32,
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
        Self {
            title: title.into(),
            url: url.into(),
            which,
            pages: vec![],
            current_p: 0,
        }
    }

    pub fn push_page(&mut self, page: Page) {
        self.pages.push(page);
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
