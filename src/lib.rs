#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;

pub mod error;
pub mod extractor;
pub mod models;

#[cfg(test)]
mod tests {
    use crate::extractor::*;
    use crate::models::*;

    #[test]
    fn test_extractor() {
        let dmjz = Dmzj {};
        let comics = dmjz.index(0).unwrap();
        assert_eq!(20, comics.len());

        let mut comic = Comic::from_link("极主夫道", "http://manhua.dmzj.com/jizhufudao/");
        dmjz.fetch_chapters(&mut comic).unwrap();
        assert_eq!(47, comic.chapters.len());

        let chapter1 = &mut comic.chapters[0];
        dmjz.fetch_pages(chapter1).unwrap();
        assert_eq!("极主夫道 第01话", chapter1.title);
        assert_eq!(16, chapter1.pages.len());
    }

    use quick_js::JsValue;

    #[test]
    fn test_js_eval() {
        match eval_as::<String>("1 + 1") {
            Ok(_) => assert!(false),
            Err(_e) => assert!(true),
        }
        let result = eval_as::<String>("(1 + 1).toString()").unwrap();
        assert_eq!("2", result);

        let value = eval_value("1 + 1").unwrap();
        assert_eq!(JsValue::Int(2), value);

        let object_code = r#"var obj = {a: 1, b: "2", c: {c1: true}, d: ["d1"]}; obj"#;
        if let JsValue::Object(json) = eval_value(object_code).unwrap() {
            assert_eq!(JsValue::Int(1), *json.get("a").unwrap());
            assert_eq!(JsValue::String(String::from("2")), *json.get("b").unwrap());
            if let JsValue::Object(c) = json.get("c").unwrap() {
                assert_eq!(JsValue::Bool(true), *c.get("c1").unwrap());
            }
            if let JsValue::Array(d) = json.get("d").unwrap() {
                for dv in d {
                    assert_eq!(JsValue::String(String::from("d1")), *dv);
                }
            }
        } else {
            assert!(false);
        }
    }
}
