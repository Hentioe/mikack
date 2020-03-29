use mikack::extractors::*;

#[test]
fn test_search() {
    let ignored_list = vec!["www.manhuagui.com".to_owned()]; // manhuagui 暂时无法访问
    let keywords = "asdfghjkl"; // 测试无结果搜索时 API 的稳定性

    for (domain, _) in platforms().iter() {
        if !ignored_list.contains(domain) {
            get_extr(domain).unwrap().search(keywords).unwrap();
        }
    }
}
