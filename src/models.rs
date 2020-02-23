use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use num_derive::FromPrimitive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub n: usize,
    pub address: String,
    pub fname: String,
    pub fmime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub url: String,
    pub which: u32,
    pub pages: Vec<Page>,
    pub page_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comic {
    pub title: String,
    pub url: String,
    pub cover: String,
    pub chapters: Vec<Chapter>,
}

macro_rules! def_tags {
    ($( {$name:tt: $str:expr} ),*,) => {
        use std::fmt;

        #[derive(PartialEq, Debug, Copy, Clone, FromPrimitive)]
        pub enum Tag {
            $(
                $name,
            )*
        }

        impl fmt::Display for Tag {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let s = match self {
                    $( Tag::$name => $str, )*
                };
                write!(f, "{}", s)
            }
        }

        impl Tag {
            #[allow(dead_code)]
            pub fn all() -> Vec<Self> {
                let mut tags = vec![];
                $( tags.push(Tag::$name); )*

                tags
            }

            #[allow(dead_code)]
            pub fn from_i32(value: i32) -> Option<Self> {
                num::FromPrimitive::from_i32(value)
            }
        }
    };
    ($( {$name:tt: $str:expr} ),*) => {
        def_tags[$( $name: $str )*,];
    }
}

def_tags![
    {Chinese: "中文"},
    {English: "英文"},
    {Japanese: "日文"},
    {NSFW: "NSFW"},
];

static DEFAULT_MIME: &'static str = "*/*";

impl Page {
    pub fn new<S: Into<String>>(n: usize, address: S) -> Self {
        let address = address.into();
        let fname = Page::fname(&address.clone(), &n);
        Self {
            n,
            address: address,
            fname,
            fmime: DEFAULT_MIME.to_string(),
        }
    }

    pub fn fname(address: &str, n: &usize) -> String {
        let mut name = n.to_string();
        if let Some(extension) = Path::new(address).extension() {
            if let Some(ext_s) = extension.to_str() {
                let params_marker_index = ext_s.find("?").unwrap_or(extension.len());
                name += &format!(".{}", ext_s[0..params_marker_index].to_string());
            }
        }
        name
    }
}

impl Chapter {
    pub fn new<S: Into<String>>(title: S, url: S, which: u32) -> Self {
        let mut page_headers = HashMap::new();
        let url = url.into();
        page_headers.insert(String::from("Referer"), url.clone());
        Self {
            title: Self::title(title),
            url: url.clone(),
            which,
            pages: vec![],
            page_headers,
        }
    }

    pub fn title<S: Into<String>>(title: S) -> String {
        title.into().trim().to_string()
    }

    pub fn set_title<S: Into<String>>(&mut self, title: S) -> &Self {
        self.title = Chapter::title(title);
        self
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
            cover: String::from(""),
            chapters: vec![],
        }
    }

    pub fn from_index<S: Into<String>>(title: S, url: S, cover: S) -> Self {
        Self {
            title: title.into(),
            url: url.into(),
            cover: cover.into(),
            chapters: vec![],
        }
    }

    pub fn push_chapter(&mut self, chapter: Chapter) {
        self.chapters.push(chapter);
    }
}

impl From<&Comic> for Chapter {
    fn from(c: &Comic) -> Self {
        Self::new(&c.title, &c.url, 0)
    }
}

pub trait FromLink {
    fn from_link<S: Into<String>>(text: S, href: S) -> Self;
}

pub trait SetCover {
    fn set_cover<S: Into<String>>(&mut self, address: S);
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

impl SetCover for Comic {
    fn set_cover<S: Into<String>>(&mut self, address: S) {
        self.cover = address.into();
    }
}

impl SetCover for Chapter {
    fn set_cover<S: Into<String>>(&mut self, _address: S) {}
}
