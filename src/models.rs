#[derive(Debug)]
pub struct Page {
    pub n: u32,
    pub address: String,
}

#[derive(Debug)]
pub struct Chapter {
    pub title: String,
    pub which: u32,
    pub pages: Vec<Page>,
    current_p: u32,
}

#[derive(Debug)]
pub struct Comic {
    pub title: String,
    pub url: String,
    pub chapters: Vec<Chapter>,
}

impl Page {
    pub fn new(n: u32, address: &str) -> Self {
        Self {
            n,
            address: address.into(),
        }
    }
}

impl Chapter {
    pub fn new(title: &str, which: u32) -> Self {
        Self {
            title: title.into(),
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
    pub fn new(title: &str, url: &str) -> Self {
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
