#[derive(Debug)]
pub struct Page {
    pub n: u32,
    pub address: String,
}

#[derive(Debug)]
pub struct Chapter {
    pub name: String,
    pub which: u32,
    pub pages: Vec<Page>,
}

#[derive(Debug)]
pub struct Comic {
    pub name: String,
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
    pub fn new(name: &str, which: u32) -> Self {
        Self {
            name: name.into(),
            which,
            pages: vec![],
        }
    }

    pub fn push_page(&mut self, page: Page) {
        self.pages.push(page);
    }
}

impl Comic {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            chapters: vec![],
        }
    }

    pub fn push_chapter(&mut self, chapter: Chapter) {
        self.chapters.push(chapter);
    }
}
