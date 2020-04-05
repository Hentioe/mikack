use num_derive::FromPrimitive;
use percent_encoding::{utf8_percent_encode, CONTROLS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{default::Default, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub n: usize,
    pub address: String,
    pub fname: String,
    pub fmime: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub url: String,
    pub which: u32,
    pub pages: Vec<Page>,
    pub page_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComicState {
    Unknown = 0,
    Completed,
    Ongoing,
}

impl Default for ComicState {
    fn default() -> Self {
        ComicState::Unknown
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Comic {
    pub title: String,
    pub url: String,
    pub cover: String,
    pub chapters: Vec<Chapter>,
    pub author: String,
    pub description: String,
    pub tags: Vec<String>,
    pub last_updated_date: i64,
    pub state: ComicState,
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
    {Wallpaper: "壁纸"},
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
    pub fn make_headers(url: &str) -> HashMap<String, String> {
        let mut page_headers = HashMap::new();
        let urlencoded = utf8_percent_encode(url, &CONTROLS).collect();
        page_headers.insert(String::from("Referer"), urlencoded);

        page_headers
    }

    pub fn new<S: Into<String>>(title: S, url: S, which: u32) -> Self {
        let url = &url.into();

        Self {
            title: Self::title(title),
            url: url.to_owned(),
            which,
            page_headers: Self::make_headers(&url),
            ..Default::default()
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
    pub fn new<T: Into<String>, U: Into<String>>(title: T, url: U) -> Self {
        Self {
            title: title.into(),
            url: url.into(),
            ..Default::default()
        }
    }

    pub fn from_index<S: Into<String>>(title: S, url: S, cover: S) -> Self {
        Self {
            title: title.into(),
            url: url.into(),
            cover: cover.into(),
            ..Default::default()
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

pub trait FromUrl {
    fn from_url<S: Into<String>>(url: S) -> Self;
}
pub trait FromLink {
    fn from_link<S: Into<String>>(text: S, href: S) -> Self;
}

pub trait SetCover {
    fn set_cover<S: Into<String>>(&mut self, address: S);
}

pub trait SetWhich {
    fn set_which(&mut self, which: usize);
}

impl FromUrl for Comic {
    fn from_url<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }
}

impl FromUrl for Chapter {
    fn from_url<S: Into<String>>(url: S) -> Self {
        let url = &url.into();
        Self {
            url: url.to_owned(),
            page_headers: Self::make_headers(url),
            ..Default::default()
        }
    }
}

impl SetWhich for Chapter {
    fn set_which(&mut self, which: usize) {
        self.which = which as u32
    }
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
