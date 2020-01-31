use crate::error::*;
use crate::models::*;
use duang::duang;
use encoding_rs::*;
use quick_js::{Context, JsValue};
use regex::Regex;
use std::any::Any;
use std::collections::HashMap;

macro_rules! def_ableitem {
    ( $(:$name:ident),* ) => {
        paste::item!{
            $(
                fn [<is_ $name>](&self) -> bool {
                    if let Some(ableitem) = self.read_state().get(format!(stringify!($name)).as_str()) {
                        ableitem.downcast_ref::<bool>() == Some(&true)
                    } else {
                        false
                    }
                }
            )*
        }
    };
}

type State = HashMap<&'static str, Box<dyn Any + Send + Sync>>;

pub trait Extractor {
    def_ableitem![:usable, :searchable, :pageable];

    fn read_state(&self) -> &State;

    fn index(&self, page: u32) -> Result<Vec<Comic>>;

    fn fetch_chapters(&self, _comic: &mut Comic) -> Result<()> {
        Ok(())
    }

    fn pages_iter<'a>(&'a self, chapter: &'a mut Chapter) -> Result<ChapterPages> {
        Ok(ChapterPages::new(
            chapter,
            0,
            vec![],
            Box::new(|_| Ok(vec![])),
        ))
    }

    fn fetch_pages(&self, chapter: &mut Chapter) -> Result<()> {
        self.pages_iter(chapter)?.for_each(drop);
        Ok(())
    }

    fn fetch_pages_unsafe(&self, chapter: &mut Chapter) -> Result<()> {
        self.pages_iter(chapter)?.for_each(|r| {
            r.unwrap();
        });
        Ok(())
    }
}

pub struct ChapterPages<'a> {
    chapter: &'a mut Chapter,
    current_page: usize,
    fetch: Box<dyn Fn(usize) -> Result<Vec<Page>>>,
    pub total: i32,
}

impl<'a> ChapterPages<'a> {
    fn new(
        chapter: &'a mut Chapter,
        total: i32,
        init_addresses: Vec<String>,
        fetch: Box<dyn Fn(usize) -> Result<Vec<Page>>>,
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
            Box::new(move |_| Ok(vec![])),
        )
    }

    #[allow(dead_code)]
    pub fn chapter_title_clone(&self) -> String {
        self.chapter.title.clone()
    }
}

impl<'a> Iterator for ChapterPages<'a> {
    type Item = Result<Page>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_page += 1;
        if self.total == 0 || (self.total > 0 && (self.total as usize) < self.current_page) {
            return None;
        }
        let page_index = self.current_page - 1;
        if ((self.chapter.pages.len() as i32) - 1) >= page_index as i32 {
            return Some(Ok(self.chapter.pages[page_index].clone()));
        }

        match (self.fetch)(self.current_page) {
            Ok(mut pages) => {
                let count = pages.len();
                self.chapter.pages.append(&mut pages);
                let current_len = self.chapter.pages.len();
                if count > 0 {
                    Some(Ok(self.chapter.pages[current_len - count].clone()))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}

use reqwest::blocking::{Client, Response};
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

use reqwest::header::USER_AGENT;

pub static DEFAULT_USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36";

pub fn get<T: reqwest::IntoUrl>(url: T) -> Result<Response> {
    Ok(Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?
        .get(url)
        .header(USER_AGENT, DEFAULT_USER_AGENT)
        .send()?)
}

fn simple_fetch_index<T: FromLink>(
    document: &Html,
    selector: &str,
    parse_elem: &dyn Fn(ElementRef) -> Result<T>,
) -> Result<Vec<T>> {
    let mut list = Vec::new();

    for element in document.select(&parse_selector(selector)?) {
        list.push(parse_elem(element)?);
    }

    Ok(list)
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

macro_rules! def_extractor {
    ( [$($name:ident: $value:expr),*], $($tt:tt)* ) => {
        pub struct Extr {
            state: State,
        }
        impl Extractor for Extr {
            $($tt)*

            fn read_state(&self) -> &State {
                &self.state
            }
        }
        impl Extr {
            fn new(state: State) -> Self {
                Self { state }
            }
        }
        pub fn new_extr() -> Extr {
            let mut state = State::new();
            $(
                state.insert(stringify!($name), Box::new($value));
            )*
            Extr::new(state)
        }
    };
    ( $($tt:tt)* ) => {
        def_extractor!{ [usable: false, searchable: false, pageable: false], $($tt)* }
    }
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
            let sub_dom_text = keyword.get_as::<&str>("sub_dom_text");
            let text_attr = keyword.get_as::<&str>("text_attr");

            let mut resp = get(url)?;
            let html = if let Some(encoding) = encoding {
                resp.decode_text(encoding)?
            } else {
                resp.text()?
            };
            let document = parse_document(&html);
            let from_link = |element: &ElementRef| -> Result<$entry> {
                let mut url = element
                    .value()
                    .attr("href")
                    .ok_or(err_msg("No link href found"))?
                    .to_string();
                if !href_prefix.is_empty() {
                    url = format!("{}{}", href_prefix, url)
                }
                let mut title = String::new();
                if let Some(sub_text_selector) = sub_dom_text {
                    let selector = parse_selector(&sub_text_selector)?;
                    title = element.select(&selector)
                        .next()
                        .ok_or(err_msg(format!("No :sub_dom_text node found: `{}`", sub_text_selector)))?
                        .text()
                        .next()
                        .ok_or(err_msg(format!("No :sub_dom_text text found: `{}`", sub_text_selector)))?
                        .to_string();
                }
                if let Some(attr) = text_attr {
                    title = element.value()
                        .attr(attr)
                        .ok_or(err_msg(format!("No :text_attr found: `{}`", attr)))?
                        .to_string();
                }
                if title.is_empty() {
                    title = element.text()
                        .next()
                        .ok_or(err_msg("No link text found"))?
                        .to_string();
                }
                title = title.trim().to_string();
                Ok($entry::from_link(title, url))
            };
            if let Some(target) = keyword.get_as::<&str>("target") {
                simple_fetch_index(&document, target, &|element: ElementRef| {
                    Ok(from_link(&element)?)
                })

            }else{
                simple_fetch_index(&document, selector, &|element: ElementRef| {
                    let link_dom = element.select(&parse_selector(&find)?)
                        .next()
                        .ok_or(err_msg(format!("No :find found: {}", find)))?;
                    Ok(from_link(&link_dom)?)
                })
            }

        }
    };
}

duang!(
    pub fn itemsgen2<T: FromLink + SetCover>(
        url: &str =  "",
        encoding: &'static Encoding = UTF_8,
        target_dom: &str = "",
        target_text_dom: &str = "",
        target_text_attr: &str = "",
        parent_dom: &str = "",
        cover_dom: &str = "",
        cover_attr: &str = "src",
        cover_prefix: &str = "",
        link_dom: &str = "",
        link_attr: &str = "href",
        link_prefix: &str = "",
        link_text_attr: &str = ""
    ) -> Result<Vec<T>> {
        if url.is_empty() {
            panic!("Missing `url` parameter");
        }
        let mut resp = get(url)?;
        let html = if encoding != UTF_8 {
            resp.decode_text(encoding)?
        } else {
            resp.text()?
        };
        let document = parse_document(&html);
        let from_link = |element: &ElementRef| -> Result<T> {
            let mut url = element
                .value()
                .attr(link_attr)
                .ok_or(err_msg("No link href found"))?
                .to_string();
            if !link_prefix.is_empty() {
                url = format!("{}{}", link_prefix, url)
            }
            let mut title = String::new();
            if !target_text_dom.is_empty() {
                title = element.select(&parse_selector(target_text_dom)?)
                    .next()
                    .ok_or(err_msg(format!("No :target_text_dom node found: `{}`", target_text_dom)))?
                    .text()
                    .next()
                    .ok_or(err_msg(format!("No :target_text_dom text found: `{}`", target_text_dom)))?
                    .to_string();
            }
            if !target_text_attr.is_empty() {
                title = element.value()
                    .attr(target_text_attr)
                    .ok_or(err_msg(format!("No :target_text_attr found: `{}`", target_text_attr)))?
                    .to_string();
            }
            if !link_text_attr.is_empty() {
                title = element.value()
                    .attr(link_text_attr)
                    .ok_or(err_msg(format!("No attr found: `{}`", link_text_attr)))?
                    .to_string();
            }
            if title.is_empty() {
                title = element
                    .text()
                    .next()
                    .ok_or(err_msg("No link text found"))?
                    .to_string();
            }
            title = title.trim().to_string();
            Ok(T::from_link(title, url))
        };

        let mut items = vec![];
        if !parent_dom.is_empty() {
            let parent_elems = document.select(&parse_selector(parent_dom)?).collect::<Vec<_>>();
            for parent_elem in parent_elems {
                let link_elem = parent_elem
                    .select(&parse_selector(link_dom)?)
                    .next()
                    .ok_or(err_msg(format!("No link DOM node found: `{}`", link_dom)))?;
                let mut item = from_link(&link_elem)?;
                let cover = parent_elem
                    .select(&parse_selector(cover_dom)?)
                    .next()
                    .ok_or(err_msg(format!("No cover DOM node found: `{}`", cover_dom)))?
                    .value()
                    .attr(cover_attr)
                    .ok_or(err_msg(format!("No cover attr found: `{}`", cover_attr)))?;
                item.set_cover(format!("{}{}", cover_prefix, cover));
                items.push(item);
            }
        } else {
            if target_dom.is_empty() {
                panic!("Missing `target_dom` parameter");
            }
            let link_elems = document.select(&parse_selector(target_dom)?).collect::<Vec<_>>();
            for link_elem in link_elems {
                items.push(from_link(&link_elem)?);
            }
        }

        Ok(items)
    }
);

trait AttachTo<T> {
    fn attach_to(self, target: &mut T);
    fn reversed_attach_to(self, target: &mut T);
}

impl AttachTo<Comic> for Vec<Chapter> {
    fn attach_to(self, target: &mut Comic) {
        for (i, mut chapter) in self.into_iter().enumerate() {
            chapter.which = (i as u32) + 1;
            target.push_chapter(chapter);
        }
    }

    fn reversed_attach_to(mut self, target: &mut Comic) {
        self.reverse();
        self.attach_to(target);
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
    eighteenh: {
        :domain => "18h.animezilla.com",
        :name   => "18H 宅宅愛動漫"
    },
    hhimm: {
        :domain => "www.hhimm.com",
        :name   => "汗汗酷漫"
    },
    kukudm: {
        :domain => "comic.kukudm.com",
        :name   => "KuKu动漫"
    },
    lhscan: {
        :domain => "lhscan.net",
        :name   => "LHScan"
    },
    luscious: {
        :domain => "www.luscious.net",
        :name   => "Luscious"
    },
    manganelo: {
        :domain => "manganelo.com",
        :name   => "Manganelo"
    },
    manhuadui: {
        :domain => "www.manhuadui.com",
        :name   => "漫画堆"
    },
    manhuadb: {
        :domain => "www.manhuadb.com",
        :name   => "漫画DB"
    },
    manhuagui: {
        :domain => "www.manhuagui.com",
        :name   => "漫画柜"
    },
    manhuaren: {
        :domain => "www.manhuaren.com",
        :name   => "漫画人"
    },
    one77pic: {
        :domain => "www.177pic.info",
        :name   => "177漫畫"
    },
    qkmh5: {
        :domain => "www.qkmh5.com",
        :name   => "青空漫画"
    },
    twoanimx: {
        :domain => "www.2animx.com",
        :name   => "二次元動漫"
    },
    veryim: {
        :domain => "comic.veryim.com",
        :name   => "非常爱漫"
    },
    xinxinmh: {
        :domain => "www.177mh.net",
        :name   => "新新漫画网"
    },
    yylsmh: {
        :domain => "8comic.se",
        :name   => "YYLS漫畫"
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
    assert!(get_extr("18h.animezilla.com").unwrap().is_usable());
    assert!(get_extr("www.hhimm.com").unwrap().is_usable());
    assert!(get_extr("comic.kukudm.com").unwrap().is_usable());
    assert!(get_extr("lhscan.net").unwrap().is_usable());
    assert!(get_extr("manganelo.com").unwrap().is_usable());
    assert!(get_extr("www.manhuadb.com").unwrap().is_usable());
    assert!(get_extr("www.manhuadui.com").unwrap().is_usable());
    assert!(get_extr("www.manhuagui.com").unwrap().is_usable());
    assert!(get_extr("www.manhuaren.com").unwrap().is_usable());
    assert!(get_extr("www.177pic.info").unwrap().is_usable());
    assert!(!get_extr("www.qkmh5.com").unwrap().is_usable());
    assert!(get_extr("www.2animx.com").unwrap().is_usable());
    assert!(!get_extr("comic.veryim.com").unwrap().is_usable());
    assert!(get_extr("www.177mh.net").unwrap().is_usable());
}

type Routes = Vec<(String, (Regex, Regex))>;

macro_rules! def_routes {
    ( $({:domain => $domain:expr, :comic_re => $comic_re:expr, :chapter_re => $chapter_re:expr}),* ) => {
        lazy_static!{
            static ref ROUTES: Routes = {
                let mut routes: Routes = Vec::new();
                $(
                    routes.push(($domain.to_string(), (Regex::new($comic_re).unwrap(), Regex::new($chapter_re).unwrap())));
                )*
                routes
            };
        }
    };
}

#[derive(Debug, PartialEq)]
pub enum DomainRoute {
    Comic(String),
    Chapter(String),
}

pub fn domain_route(url: &str) -> Option<DomainRoute> {
    for (domain, (comic_re, chapter_re)) in &*ROUTES {
        if chapter_re.is_match(url) {
            return Some(DomainRoute::Chapter(domain.clone()));
        }
        if comic_re.is_match(url) {
            return Some(DomainRoute::Comic(domain.clone()));
        }
    }
    None
}

def_routes![
    {
        :domain     => "www.cartoonmad.com",
        :comic_re   => r#"^https?://www\.cartoonmad\.com/comic/\d{1,5}\.html"#,
        :chapter_re => r#"^https?://www\.cartoonmad\.com/comic/\d{11,}.html"#
    },
    {
        :domain     => "www.dm5.com",
        :comic_re   => r#"^https?://www\.dm5\.com/[^/]+/"#,
        :chapter_re => r#"^https?://www\.dm5\.com/m\d+/"#
    },
    {
        :domain     => "manhua.dmzj.com",
        :comic_re   => r#"^https?://manhua\.dmzj\.com/[^/]+/"#,
        :chapter_re => r#"^https?://manhua\.dmzj\.com/[^/]+/\d+\.shtml"#
    },
    {
        :domain     => "e-hentai.org",
        :comic_re   => r#"^-NONE-$"#,
        :chapter_re => r#"^https?://e-hentai\.org/g/\d+/[^/]+/"#
    },
    {
        :domain     => "18h.animezilla.com",
        :comic_re   => r#"^-NONE-$"#,
        :chapter_re => r#"^https?://18h\.animezilla\.com/manga/\d+"#
    },
    {
        :domain     => "www.hhimm.com",
        :comic_re   => r#"^https?://www\.hhimm\.com/manhua/\d+\.html"#,
        :chapter_re => r#"^https?://www\.hhimm\.com/cool\d+/\d+\.html"#
    },
    {
        :domain     => "comic.kukudm.com",
        :comic_re   => r#"^https?://comic\.kukudm\.com/comiclist/\d+/index.htm"#,
        :chapter_re => r#"^https?://comic\d?\.kukudm\.com/comiclist/\d+/\d+/\d+.htm"#
    },
    {
        :domain     => "lhscan.net",
        :comic_re   => r#"^https?://lhscan\.net/manga-.+\.html"#,
        :chapter_re => r#"^https?://lhscan\.net/read-.+\.html"#
    },
    {
        :domain     => "www.luscious.net",
        :comic_re   => r#"^-NONE-$"#,
        :chapter_re => r#"^https?://www\.luscious\.net/albums/.+"#
    },
    {
        :domain     => "manganelo.com",
        :comic_re   => r#"^https?://manganelo\.com/manga/.+"#,
        :chapter_re => r#"^https?://manganelo\.com/chapter/[^/]+/chapter_.+"#
    },
    {
        :domain     => "www.manhuadb.com",
        :comic_re   => r#"^https?://www\.manhuadb\.com/manhua/.+"#,
        :chapter_re => r#"^https?://www\.manhuadb\.com/manhua/\d+/\d+_\d+\.html"#
    },
    {
        :domain     => "www.manhuadui.com",
        :comic_re   => r#"^https?://www\.manhuadui\.com/manhua/.+"#,
        :chapter_re => r#"^https?://www\.manhuadui\.com/manhua/[^/]+/\d+\.html"#
    },
    {
        :domain     => "www.manhuagui.com",
        :comic_re   => r#"^https?://www\.manhuagui\.com/comic/\d+/"#,
        :chapter_re => r#"^https?://www\.manhuagui\.com/comic/\d+/\d+\.html"#
    },
    {
        :domain     => "www.manhuaren.com",
        :comic_re   => r#"^https?://www\.manhuaren\.com/manhua-[^/]+/"#,
        :chapter_re => r#"^https?://www\.manhuaren\.com/m\d+/"#
    },
    {
        :domain     => "www.177pic.info",
        :comic_re   => r#"^-NONE-$"#,
        :chapter_re => r#"^https?://www\.177pic\.info/html/\d+/\d+/\d+\.html"#
    },
    {
        :domain     => "www.qkmh5.com",
        :comic_re   => r#"^https?://www\.qkmh5\.com/mh/[^\.]+\.html"#,
        :chapter_re => r#"^https?://www\.qkmh5\.com/mm/\d+/\d+\.html"#
    },
    {
        :domain     => "www.2animx.com",
        :comic_re   => r#"^https?://www\.2animx\.com/index-comic-name-.+-id-\d+"#,
        :chapter_re => r#"^https?://www\.2animx\.com/index-look-name-.+-cid-\d+-id-\d+"#
    },
    {
        :domain     => "comic.veryim.com",
        :comic_re   => r#"^https?://comic\.veryim\.com/[^/]+/\d+/"#,
        :chapter_re => r#"^https?://comic\.veryim\.com/[^/]+/\d+/\d+\.html"#
    },
    {
        :domain     => "www.177mh.net",
        :comic_re   => r#"^https?://www\.177mh\.net/colist_\d+\.html"#,
        :chapter_re => r#"^https?://www.177mh.net/\d+/\d+\.html"#
    },
    {
        :domain     => "8comic.se",
        :comic_re   => r#"^-NONE-$"#,
        :chapter_re => r#"^https?://8comic\.se/\d+"#
    }
];

#[test]
fn test_routes() {
    assert_eq!(
        DomainRoute::Comic(String::from("www.cartoonmad.com")),
        domain_route("https://www.cartoonmad.com/comic/8460.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("www.cartoonmad.com")),
        domain_route("https://www.cartoonmad.com/comic/846000012038001.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("www.dm5.com")),
        domain_route("http://www.dm5.com/manhua-yuanzun/").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("www.dm5.com")),
        domain_route("http://www.dm5.com/m578500/").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("manhua.dmzj.com")),
        domain_route("http://manhua.dmzj.com/yifuyaozhemechuan/").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("manhua.dmzj.com")),
        domain_route("http://manhua.dmzj.com/yifuyaozhemechuan/56275.shtml#@page=1").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("e-hentai.org")),
        domain_route("https://e-hentai.org/g/1552929/c9f7a6ad71/").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("18h.animezilla.com")),
        domain_route("https://18h.animezilla.com/manga/2940").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("www.hhimm.com")),
        domain_route("http://www.hhimm.com/manhua/40325.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("www.hhimm.com")),
        domain_route("http://www.hhimm.com/cool373925/1.html?s=3").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("comic.kukudm.com")),
        domain_route("https://comic.kukudm.com/comiclist/2555/index.htm").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("comic.kukudm.com")),
        domain_route("https://comic.kukudm.com/comiclist/2555/66929/1.htm").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("comic.kukudm.com")),
        domain_route("https://comic2.kukudm.com/comiclist/2555/66929/1.htm").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("lhscan.net")),
        domain_route("https://lhscan.net/manga-ichinichi-gaishutsuroku-hanchou-raw.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("lhscan.net")),
        domain_route("https://lhscan.net/read-ichinichi-gaishutsuroku-hanchou-raw-chapter-54.html")
            .unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("www.luscious.net")),
        domain_route("https://www.luscious.net/albums/teitoku-wa-semai-toko-suki-kantai-collection-kanco_363520/")
            .unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("manganelo.com")),
        domain_route("https://manganelo.com/manga/hgj2047065412").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("manganelo.com")),
        domain_route("https://manganelo.com/chapter/hgj2047065412/chapter_43").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("www.manhuadb.com")),
        domain_route("https://www.manhuadb.com/manhua/10906").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("www.manhuadb.com")),
        domain_route("https://www.manhuadb.com/manhua/10906/13071_183254.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("www.manhuadui.com")),
        domain_route("https://www.manhuadui.com/manhua/jingjiechufazhe/").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("www.manhuadui.com")),
        domain_route("https://www.manhuadui.com/manhua/jingjiechufazhe/435634.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("www.manhuagui.com")),
        domain_route("https://www.manhuagui.com/comic/20515/").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("www.manhuagui.com")),
        domain_route("https://www.manhuagui.com/comic/20515/469245.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("www.manhuaren.com")),
        domain_route("https://www.manhuaren.com/manhua-fengyunquanji/").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("www.manhuaren.com")),
        domain_route("https://www.manhuaren.com/m188947/").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("www.177pic.info")),
        domain_route("http://www.177pic.info/html/2020/01/3307768.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("www.qkmh5.com")),
        domain_route("http://www.qkmh5.com/mh/yaojingdeweiba.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("www.qkmh5.com")),
        domain_route("http://www.qkmh5.com/mm/1807/461806.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("www.2animx.com")),
        domain_route(
            "http://www.2animx.com/index-comic-name-%E9%A2%A8%E9%9B%B2%E5%85%A8%E9%9B%86-id-7212"
        )
        .unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("www.2animx.com")),
        domain_route("http://www.2animx.com/index-look-name-%E9%A2%A8%E9%9B%B2%E5%85%A8%E9%9B%86-cid-7212-id-88034").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("comic.veryim.com")),
        domain_route("http://comic.veryim.com/qihuan/57238/").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("comic.veryim.com")),
        domain_route("http://comic.veryim.com/qihuan/57238/883902.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Comic(String::from("www.177mh.net")),
        domain_route("https://www.177mh.net/colist_244241.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("www.177mh.net")),
        domain_route("https://www.177mh.net/202001/437290.html").unwrap()
    );
    assert_eq!(
        DomainRoute::Chapter(String::from("8comic.se")),
        domain_route("http://8comic.se/879/").unwrap()
    );
}
