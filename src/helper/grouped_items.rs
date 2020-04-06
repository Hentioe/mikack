use super::*;
use crate::models::{FromLink, SetWhich};
use scraper::Html;
use std::default::Default;
pub use std::rc::Rc;

pub struct GroupedItemsSelector<'a> {
    pub document: Rc<Html>,
    pub group_dom: &'a str,
    pub inside_group_name_dom: &'a str,
    pub outside_group_name_dom: &'a str,
    pub items_dom: &'a str,
    pub items_title_attr: &'a str,
    pub items_title_dom: &'a str,
    pub items_title_dom_attr: &'a str,
    pub items_url_attr: &'a str,
    pub items_url_prefix: &'a str,
}

type GroupedItemsType<T> = (String, Vec<T>);

impl<'a> GroupedItemsSelector<'a> {
    pub fn gen<T: FromLink>(&self) -> Result<Vec<GroupedItemsType<T>>> {
        let mut grouped_items = vec![];
        let outide_names = if !self.outside_group_name_dom.is_empty() {
            let mut names = vec![];
            for (i, name_elem) in self
                .document
                .select(&parse_selector(self.outside_group_name_dom)?)
                .enumerate()
            {
                names.push(name_elem.text().next().ok_or(err_msg(format!(
                    "No group name text found at location `{}`",
                    i
                )))?);
            }
            names
        } else {
            vec![]
        };
        for (i, group) in self
            .document
            .select(&parse_selector(self.group_dom)?)
            .enumerate()
        {
            let gname = if !self.inside_group_name_dom.is_empty() {
                // 设置组名
                if let Some(item) = group
                    .select(&parse_selector(self.inside_group_name_dom)?)
                    .next()
                {
                    item.text()
                        .next()
                        .ok_or(err_msg(format!(
                            "No group name `{}` found at location `{}`",
                            self.inside_group_name_dom, i
                        )))?
                        .to_string()
                } else {
                    format!("第{}组", i + 1)
                }
            } else {
                // 根据位置获取外部分组名称
                if let Some(name) = outide_names.get(i) {
                    name.to_string()
                } else {
                    format!("第{}组", i + 1)
                }
            };
            let mut items = vec![];
            for (i, item) in group.select(&parse_selector(self.items_dom)?).enumerate() {
                let title = if !self.items_title_attr.is_empty() {
                    // 取指定属性值
                    item.value()
                        .attr(self.items_title_attr)
                        .ok_or(err_msg(format!(
                            "No item title attribute `{}` found at location `{}`",
                            self.items_title_attr, i
                        )))?
                } else if !self.items_title_dom.is_empty() {
                    let title_dom = item
                        .select(&parse_selector(self.items_title_dom)?)
                        .next()
                        .ok_or(err_msg(format!("No title dom found at location `{}`", i)))?;
                    if self.items_title_dom_attr.is_empty() {
                        // 直接取标题 dom 文本
                        title_dom.text().next().ok_or(err_msg(format!(
                            "No item title dom text found at location `{}`",
                            i
                        )))?
                    } else {
                        // 直接取标题 dom 指定属性值
                        title_dom
                            .value()
                            .attr(self.items_title_dom_attr)
                            .ok_or(err_msg(format!(
                                "No item title dom attribute `{}` found at location `{}`",
                                self.items_title_dom_attr, i
                            )))?
                    }
                } else {
                    // 直接取文本
                    item.text().next().ok_or(err_msg(format!(
                        "No item title text found at location `{}`",
                        i
                    )))?
                };
                let mut url = item
                    .value()
                    .attr(self.items_url_attr)
                    .ok_or(err_msg(format!(
                        "No item url attribute `{}` found at location `{}`",
                        self.items_url_attr, i
                    )))?
                    .to_string();
                if !self.items_url_prefix.is_empty() {
                    url = self.items_url_prefix.to_owned() + &url;
                }
                items.push(T::from_link(title.trim(), url.trim()))
            }

            grouped_items.push((gname, items));
        }

        Ok(grouped_items)
    }
}

pub trait Flatten<T> {
    fn flatten(&mut self, group_index: usize) -> Vec<T>;

    fn reversed_flatten(&mut self, group_index: usize) -> Vec<T>;
}

const GROUP_SPACING: usize = 10000;

impl<T: FromLink + SetWhich + Clone> Flatten<T> for Vec<GroupedItemsType<T>> {
    fn flatten(&mut self, group_index: usize) -> Vec<T> {
        let mut items = vec![];
        let mut current_count = 0;
        for group in self.iter_mut() {
            for (i, item) in group.1.iter_mut().enumerate() {
                item.set_which(GROUP_SPACING * group_index + current_count + i + 1);
                items.push(item.clone());
            }
            current_count += group.1.len();
        }

        items
    }

    fn reversed_flatten(&mut self, group_index: usize) -> Vec<T> {
        let mut items = vec![];
        let mut current_count = 0;
        for group in self.iter_mut() {
            group.1.reverse();
            for (i, item) in group.1.iter_mut().enumerate() {
                item.set_which(GROUP_SPACING * group_index + current_count + i + 1);
                items.push(item.clone());
            }
            current_count += group.1.len();
        }

        items
    }
}

impl Default for GroupedItemsSelector<'_> {
    fn default() -> Self {
        Self {
            document: Rc::new(Html::parse_document("")),
            group_dom: Default::default(),
            inside_group_name_dom: Default::default(),
            outside_group_name_dom: Default::default(),
            items_dom: Default::default(),
            items_title_attr: Default::default(),
            items_title_dom: Default::default(),
            items_title_dom_attr: Default::default(),
            items_url_attr: "href",
            items_url_prefix: Default::default(),
        }
    }
}
