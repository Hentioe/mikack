use mikack::extractors::*;

#[test]
fn test_search() {
    let ignored_list = vec![];
    let only_includes: Vec<String> = vec![];
    let keywords = "asdfghjkl"; // 测试无结果搜索时 API 的稳定性

    for (domain, _) in platforms().iter() {
        if only_includes.is_empty() {
            if !ignored_list.contains(domain) {
                get_extr(domain)
                    .unwrap()
                    .paginated_search(keywords, 1)
                    .unwrap();
            }
        } else {
            if only_includes.contains(domain) {
                get_extr(domain)
                    .unwrap()
                    .paginated_search(keywords, 1)
                    .unwrap();
            }
        }
    }
}
