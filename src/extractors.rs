use encoding_rs::*;
use quick_js::{Context, JsValue};
use regex::Regex;
use std::collections::HashMap;

use crate::error::*;
use crate::models::*;

pub trait Extractor {
    fn is_usable(&self) -> bool {
        true
    }

    fn index(&self, page: u32) -> Result<Vec<Comic>>;

    fn fetch_chapters(&self, _comic: &mut Comic) -> Result<()> {
        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        Ok(ChapterPages::new(chapter, 0, vec![], Box::new(|_| vec![])))
    }

    fn fetch_pages(&self, chapter: &mut Chapter) -> Result<()> {
        self.pages_iter(chapter)?.for_each(drop);
        Ok(())
    }
}

pub struct ChapterPages<'a> {
    chapter: &'a mut Chapter,
    current_page: usize,
    fetch: Box<dyn Fn(usize) -> Vec<Page>>,
    total: i32,
}

impl<'a> ChapterPages<'a> {
    fn new(
        chapter: &'a mut Chapter,
        total: i32,
        init_addresses: Vec<String>,
        fetch: Box<dyn Fn(usize) -> Vec<Page>>,
    ) -> Self {
        for (i, address) in init_addresses.iter().enumerate() {
            chapter.pages.push(Page::new(i as usize, address));
        }
        ChapterPages {
            chapter,
            current_page: 0,
            fetch,
            total,
        }
    }

    fn full(chapter: &'a mut Chapter, addresses: Vec<String>) -> Self {
        Self::new(
            chapter,
            addresses.len() as i32,
            addresses,
            Box::new(move |_| vec![]),
        )
    }
}

impl<'a> Iterator for ChapterPages<'a> {
    type Item = Page;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_page += 1;
        if self.total == 0 || (self.total > 0 && (self.total as usize) < self.current_page) {
            return None;
        }
        let page_index = self.current_page - 1;
        if ((self.chapter.pages.len() as i32) - 1) >= page_index as i32 {
            return Some(self.chapter.pages[page_index].clone());
        }

        let mut pages = (self.fetch)(self.current_page);
        let count = pages.len();
        self.chapter.pages.append(&mut pages);
        let current_len = self.chapter.pages.len();

        if count > 0 {
            Some(self.chapter.pages[current_len - count + 1].clone())
        } else {
            None
        }
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

trait HtmlHelper {
    fn dom_text(&self, selector: &str) -> Result<String>;
    fn dom_attrs(&self, selector: &str, attr: &str) -> Result<Vec<String>>;
    fn dom_attr(&self, selector: &str, attr: &str) -> Result<String> {
        let attrs = self.dom_attrs(selector, attr)?;
        if attrs.len() == 0 {
            Err(err_msg(format!("DOM node not found: {}", selector)))
        } else {
            Ok(attrs[0].clone())
        }
    }
}

impl HtmlHelper for Html {
    fn dom_text(&self, selector: &str) -> Result<String> {
        let st = parse_selector(selector)?;
        let dom = self
            .select(&st)
            .next()
            .ok_or(err_msg(format!("DOM node not found: {}", selector)))?;
        let text = dom
            .text()
            .next()
            .ok_or(err_msg(format!("DOM text not found: {}", selector)))?
            .trim()
            .to_string();

        Ok(text)
    }

    fn dom_attrs(&self, selector: &str, attr: &str) -> Result<Vec<String>> {
        let mut attrs = vec![];

        for element in self.select(&parse_selector(selector)?) {
            let attr_s = element.value().attr(&attr).ok_or(err_msg(format!(
                "Attribute `{}` not found in `{}`",
                attr, selector
            )))?;
            attrs.push(attr_s.to_string());
        }

        Ok(attrs)
    }
}

fn simple_fetch_index<T: FromLink>(
    html: String,
    selector: &str,
    parse_elem: &dyn Fn(ElementRef) -> Result<T>,
) -> Result<Vec<T>> {
    let document = parse_document(&html);
    let mut list = Vec::new();

    for element in document.select(&parse_selector(selector)?) {
        list.push(parse_elem(element)?);
    }

    Ok(list)
}

fn simple_parse_link(element: ElementRef, selector: Option<&str>) -> Result<(String, String)> {
    let link_elem = if let Some(selector) = selector {
        element
            .select(&parse_selector(selector)?)
            .next()
            .ok_or(err_msg("No link found"))?
    } else {
        element
    };
    let title = link_elem
        .text()
        .next()
        .ok_or(err_msg("No link text found"))?;
    let url = link_elem
        .value()
        .attr("href")
        .ok_or(err_msg("No link href found"))?;

    Ok((title.to_string(), url.to_string()))
}

pub fn eval_value(code: &str) -> Result<JsValue> {
    let context = Context::new()?;
    Ok(context.eval(code)?)
}

pub fn eval_as<R>(code: &str) -> Result<R>
where
    R: std::convert::TryFrom<JsValue>,
    R::Error: std::convert::Into<quick_js::ValueError>,
{
    let context = Context::new()?;
    Ok(context.eval_as::<R>(code)?)
}

trait Decode {
    fn decode_text(&mut self, encoding: &'static Encoding) -> Result<String>;
}

impl Decode for reqwest::blocking::Response {
    fn decode_text(&mut self, encoding: &'static Encoding) -> Result<String> {
        let mut buf: Vec<u8> = vec![];
        self.copy_to(&mut buf)?;
        let (cow, _encoding_used, _had_errors) = encoding.decode(&buf);
        Ok(cow[..].to_string())
    }
}

type JsObject = HashMap<String, JsValue>;

pub fn eval_as_obj(code: &str) -> Result<JsObject> {
    match eval_value(code)? {
        JsValue::Object(obj) => Ok(obj),
        _ => Err(err_msg("Not a JS Object")),
    }
}

macro_rules! def_js_helper {
    ( to_object: [$( {:name => $name:ident, :js_t => $js_t:path, :result_t => $result_t:ty} ),*] ) => {
        trait JsObjectGetAndAs {
            $(
                fn $name(&self, key: &str) -> Result<&$result_t>;
            )*
        }
        impl JsObjectGetAndAs for JsObject {
            $(
                fn $name(&self, key: &str) -> Result<&$result_t> {
                    let value = self
                                .get(key)
                                .ok_or(err_msg(format!("Object property not found: {}", key)))?;
                    match value {
                        $js_t(v) => Ok(v),
                        _ => Err(err_msg(format!("Object property `{}` is not of type `{}`", key, stringify!($js_t))))
                    }
                }
            )*
        }
    };
    ( to_value: [$( {:name => $name:ident, :js_t => $js_t:path, :result_t => $result_t:ty} ),*] ) => {
        trait JsValueAs {
            $(
                fn $name(&self) -> Result<&$result_t>;
            )*
        }
        impl JsValueAs for JsValue {
            $(
                fn $name(&self) -> Result<&$result_t> {
                    match self {
                        $js_t(v) => Ok(v),
                        _ => Err(err_msg(format!("Object property is not of type `{}`", stringify!($js_t))))
                    }
                }
            )*
        }
    };
}

def_js_helper!(to_object: [
    {:name => get_as_string, :js_t => JsValue::String, :result_t => String},
    {:name => get_as_bool, :js_t => JsValue::Bool, :result_t => bool},
    {:name => get_as_int, :js_t => JsValue::Int, :result_t => i32},
    {:name => get_as_float, :js_t => JsValue::Float, :result_t => f64},
    {:name => get_as_array, :js_t => JsValue::Array, :result_t => Vec<JsValue>},
    {:name => get_as_object, :js_t => JsValue::Object, :result_t => JsObject}
]);

def_js_helper!(to_value: [
    {:name => as_string, :js_t => JsValue::String, :result_t => String},
    {:name => as_bool, :js_t => JsValue::Bool, :result_t => bool},
    {:name => as_int, :js_t => JsValue::Int, :result_t => i32},
    {:name => as_float, :js_t => JsValue::Float, :result_t => f64},
    {:name => as_array, :js_t => JsValue::Array, :result_t => Vec<JsValue>},
    {:name => as_object, :js_t => JsValue::Object, :result_t => JsObject}
]);

macro_rules! def_exctractor {
    ( $( $tt:tt )* ) => {
        pub struct Extr;
        impl Extractor for Extr {
            $($tt)*
        }
        impl Extr {
            fn new() -> Self {
                Self {}
            }
        }
        pub fn new_extr() -> Extr {
            Extr::new()
        }
    };
}

macro_rules! keyword_list {
    ( $( :$name:ident => $value:expr ),* ) => {
        {
            let keyword: HashMap<&str, &dyn std::any::Any> = std::collections::HashMap::new();
            $(
                keyword.insert(stringify!($name), $value);
            )*
            keyword
        }
    };
}

macro_rules! keyword_fetch {
    ($keyword:expr, $key:expr, $t:ty, $default:expr) => {
        if let Some(v) = $keyword.get($key) {
            v.downcast_ref::<$t>().unwrap_or($default)
        } else {
            $default
        }
    };
}

trait Keyword {
    fn get_as<T>(&self, key: &str) -> Option<&T>
    where
        T: 'static;
}

impl Keyword for HashMap<&str, &dyn std::any::Any> {
    fn get_as<T>(&self, key: &str) -> Option<&T>
    where
        T: 'static,
    {
        if let Some(v) = self.get(key) {
            v.downcast_ref::<T>()
        } else {
            None
        }
    }
}

macro_rules! urlgen {
    ( $( :$name:ident => $value:expr ),* ) => {
        {
            let mut keyword = keyword_list![];
            $(
                keyword.insert(stringify!($name), $value);
            )*
            let first = keyword_fetch!(keyword, "first", &str, &"");
            let next = keyword_fetch!(keyword, "next", &str, &"");
            let page = keyword_fetch!(keyword, "page", u32, &0_u32);

            if *page > 1 {
                next.replace("{}", &page.to_string())
            } else {
                first.to_string()
            }
        }
    };
}

lazy_static! {
    pub static ref DEFAULT_STRING: String = "".to_string();
    pub static ref DEFAULT_REGEX: Regex = Regex::new("^_n_o_r_e_$").unwrap();
    pub static ref DEFAULT_FETCHING_FN: Box<dyn Fn(usize) -> Vec<String> + Sync + Send> =
        Box::new(|_| vec![]);
}

macro_rules! itemsgen {
    ( :entry => $entry:tt, $( :$name:ident => $value:expr ),* ) => {
        {
            let mut keyword = keyword_list![];
            $(
                keyword.insert(stringify!($name), $value);
            )*

            let url = keyword_fetch!(keyword, "url", String, &*DEFAULT_STRING);
            let selector = keyword_fetch!(keyword, "selector", &str, &"");
            let find = keyword_fetch!(keyword, "find", &str, &"a");
            let href_prefix = keyword_fetch!(keyword, "href_prefix", &str, &"");
            let encoding = keyword.get_as::<&Encoding>("encoding");

            let mut resp = get(url)?;
            let html = if let Some(encoding) = encoding {
                resp.decode_text(encoding)?
            }else{
                resp.text()?
            };

            if let Some(target) = keyword.get_as::<&str>("target") {
                simple_fetch_index(html, target, &|element: ElementRef| {
                    let (mut title, mut url) = simple_parse_link(element, None)?;
                    if !href_prefix.is_empty() {
                        url = format!("{}{}", href_prefix, url)
                    }
                    title = title.trim().to_string();
                    Ok($entry::from_link(title, url))
                })

            }else{
                simple_fetch_index(html, selector, &|element: ElementRef| {
                    let (mut title, mut url) = simple_parse_link(element, Some(find))?;
                    if !href_prefix.is_empty() {
                        url = format!("{}{}", href_prefix, url)
                    }
                    title = title.trim().to_string();
                    Ok($entry::from_link(title, url))
                })
            }

        }
    };
}

trait AttachTo<T> {
    fn attach_to(self, target: &mut T);
}

impl AttachTo<Comic> for Vec<Chapter> {
    fn attach_to(self, target: &mut Comic) {
        for (i, mut chapter) in self.into_iter().enumerate() {
            chapter.which = (i as u32) + 1;
            target.push_chapter(chapter);
        }
    }
}

macro_rules! def_regex {
    ( $( $name:ident => $expr:expr ),* ) => {
        $(
            lazy_static! {
                static ref $name: Regex = Regex::new($expr).unwrap();
            }
        )*
    };
}

macro_rules! match_content {
    ( $( :$name:ident => $value:expr ),* ) => {
        {
            let mut keyword = keyword_list![];
            $(
                keyword.insert(stringify!($name), $value);
            )*

            let text = keyword_fetch!(keyword, "text", String, &*DEFAULT_STRING);
            let re = keyword_fetch!(keyword, "regex", Regex, &*DEFAULT_REGEX);
            let group = keyword_fetch!(keyword, "group", usize, &1);
            let caps = re.captures(text)
                .ok_or(err_msg("No crypro code found"))?;

            caps.get(*group)
                .ok_or(err_msg("No crypro code found"))?
                .as_str()
        }
    };
}

macro_rules! wrap_code {
    ($code:expr, $custom:expr, :end) => {
        format!("{}\n{}", $code, $custom);
    };
}

#[test]
fn test_eval_as() {
    match eval_as::<String>("1 + 1") {
        Ok(_) => assert!(false),
        Err(_e) => assert!(true),
    }
    let result = eval_as::<String>("(1 + 1).toString()").unwrap();
    assert_eq!("2", result);
}

#[test]
fn test_eval_value() {
    let value = eval_value("1 + 1").unwrap();
    assert_eq!(JsValue::Int(2), value);
}

#[test]
fn test_eval_obj() {
    let code = r#"
        var obj = {
            a: 1,
            b: "b",
            c: {
                c1: true
            },
            d: ["d1"]
        };
        obj
    "#;
    let obj = eval_as_obj(&code).unwrap();
    assert_eq!(1, *obj.get_as_int("a").unwrap());
    assert_eq!(String::from("b"), *obj.get_as_string("b").unwrap());

    let c = obj.get_as_object("c").unwrap();
    assert_eq!(true, *c.get_as_bool("c1").unwrap());

    let d = obj.get_as_array("d").unwrap();
    assert_eq!(1, d.len());
    assert_eq!(String::from("d1"), *d[0].as_string().unwrap());
}

type ExtractorObject = Box<dyn Extractor + Sync + Send>;

macro_rules! import_impl_mods {
    ( $($module:ident: {:domain => $domain:expr, :name => $name:expr}),* ) => {
        $(
            pub mod $module;
        )*
        lazy_static!{
            pub static ref PLATFORMS: HashMap<String, String> = {
                let mut platforms = HashMap::new();
                $(
                    platforms.insert($domain.to_string(), $name.to_string());
                )*
                platforms
            };

            pub static ref EXTRACTORS: HashMap<String, ExtractorObject> = {
                let mut extractros: HashMap<String, ExtractorObject> = HashMap::new();
                $(
                    extractros.insert($domain.to_string(), Box::new($module::new_extr()));
                )*
                extractros
            };
        }
    }
}

import_impl_mods![
    cartoonmad: {
        :domain => "www.cartoonmad.com",
        :name   => "動漫狂"
    },
    dm5: {
        :domain => "www.dm5.com",
        :name   => "动漫屋"
    },
    dmjz: {
        :domain => "manhua.dmzj.com",
        :name   => "动漫之家"
    },
    ehentai: {
        :domain => "e-hentai.org",
        :name   => "E-Hentai"
    },
    hhimm: {
        :domain => "www.hhimm.com",
        :name   => "汗汗酷漫"
    },
    kukudm: {
        :domain => "comic.kukudm.com",
        :name   => "KuKu动漫"
    },
    manhuagui: {
        :domain => "www.manhuagui.com",
        :name   => "看漫画"
    },
    manhuaren: {
        :domain => "www.manhuaren.com",
        :name   => "漫画人"
    },
    qkmh5: {
        :domain => "www.qkmh5.com",
        :name   => "青空漫画"
    },
    veryim: {
        :domain => "comic.veryim.com",
        :name   => "非常爱漫"
    },
    xinxinmh: {
        :domain => "www.177mh.net",
        :name   => "新新漫画网"
    }
];

pub fn get_extr<S: Into<String>>(domain: S) -> Option<&'static ExtractorObject> {
    EXTRACTORS.get(&domain.into())
}

#[test]
fn test_usable() {
    assert!(get_extr("www.cartoonmad.com").unwrap().is_usable());
    assert!(get_extr("www.dm5.com").unwrap().is_usable());
    assert!(get_extr("manhua.dmzj.com").unwrap().is_usable());
    assert!(get_extr("e-hentai.org").unwrap().is_usable());
    assert!(get_extr("www.hhimm.com").unwrap().is_usable());
    assert!(get_extr("comic.kukudm.com").unwrap().is_usable());
    assert!(get_extr("www.manhuagui.com").unwrap().is_usable());
    assert!(get_extr("www.manhuaren.com").unwrap().is_usable());
    assert!(get_extr("www.qkmh5.com").unwrap().is_usable());
    assert!(!get_extr("comic.veryim.com").unwrap().is_usable());
    assert!(get_extr("www.177mh.net").unwrap().is_usable());
}
