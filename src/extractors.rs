use quick_js::{Context, JsValue};
use regex::Regex;
use std::collections::HashMap;

use crate::error::*;
use crate::models::*;

trait Extractor {
    fn index(&self, page: u32) -> Result<Vec<Comic>>;
    fn fetch_chapters(&self, _comic: &mut Comic) -> Result<()> {
        Ok(())
    }
    fn fetch_pages(&self, _chapter: &mut Chapter) -> Result<()> {
        Ok(())
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

fn simple_fetch_index<T: FromLink>(
    url: &str,
    selector: &str,
    parse_elem: &dyn Fn(ElementRef) -> Result<T>,
) -> Result<Vec<T>> {
    let html = get(url)?.text()?;
    let document = parse_document(&html);
    let mut list = Vec::new();

    for element in document.select(&parse_selector(selector)?) {
        list.push(parse_elem(element)?);
    }

    Ok(list)
}

fn simple_parse_link(element: ElementRef, selector: &str) -> Result<(String, String)> {
    let link_elem = element
        .select(&parse_selector(selector)?)
        .next()
        .ok_or(err_msg("No link found"))?;
    let title = link_elem.text().next().ok_or(err_msg("No title found"))?;
    let url = link_elem
        .value()
        .attr("href")
        .ok_or(err_msg("No href found"))?;

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
            let keyword: std::collections::HashMap<&str, &dyn std::any::Any> = std::collections::HashMap::new();
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

            if *page > 0 {
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

            simple_fetch_index(url, selector, &|element: ElementRef| {
                let (title, url) = simple_parse_link(element, find)?;
                Ok($entry::from_link(title, url))
            })
        }
    };
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

macro_rules! match_code {
    ( $( :$name:ident => $value:expr ),* ) => {
        {
            let mut keyword = keyword_list![];
            $(
                keyword.insert(stringify!($name), $value);
            )*

            let html = keyword_fetch!(keyword, "html", String, &*DEFAULT_STRING);
            let re = keyword_fetch!(keyword, "regex", Regex, &*DEFAULT_REGEX);
            let group = keyword_fetch!(keyword, "group", usize, &1);
            let caps = re.captures(html)
                .ok_or(err_msg("No crypro code found"))?;

            caps.get(*group)
                .ok_or(err_msg("No crypro code found"))?
                .as_str()
        }
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

pub mod dmjz;
