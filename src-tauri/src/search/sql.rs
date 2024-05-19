use itertools::Itertools;

use super::{ParsedKeyword, ParsedMeta, ParsedTag, Search};

pub fn build_search_sql_stmt(search: Search, page: i32, page_size: i32) -> String {
    let Search {
        tags,
        keywords,
        meta,
    } = search;
    let mut stmt = if tags.is_empty() {
        build_main_stem("sticky", &keywords, &meta)
    } else {
        format!(
            "WITH taged_sticky AS ({}) {}",
            build_sub_query_tages(&tags).unwrap(),
            build_main_stem("taged_sticky", &keywords, &meta)
        )
    };
    stmt.push_str(&format!(" LIMIT {} OFFSET {}", page_size, page * page_size));
    stmt.push(';');
    stmt
}

fn build_main_stem(
    src_table: &str,
    keywords: &Vec<ParsedKeyword>,
    meta: &Vec<ParsedMeta>,
) -> String {
    let mut sources = Vec::new();
    let mut cond = Vec::new();

    sources.push(format!("{} as inp", src_table));

    for kwd in keywords {
        cond.push(format!(
            "inp.name {} '%{}%'",
            if kwd.not { "NOT LIKE" } else { "LIKE" },
            kwd.value
        ))
    }

    for m in meta.iter().unique_by(|m| m.key) {
        match m.key {
            super::MetaKey::DateStart => {
                cond.push(format!("inp.modify_date <= {}", m.value));
            }
            super::MetaKey::DateEnd => {
                cond.push(format!("inp.modify_date >= {}", m.value));
            }
            super::MetaKey::Package => {
                sources.push(String::from("JOIN package on inp.package = package.id"));
                cond.push(format!("package.name = '{}'", m.value));
            }
        };
    }

    format!(
        "SELECT inp.* FROM {}{}",
        sources
            .into_iter()
            .reduce(|pre, acc| format!("{} {}", pre, acc))
            .unwrap(),
        cond.into_iter()
            .reduce(|pre, acc| format!("{} AND {}", pre, acc))
            .map(|c| format!(" WHERE {}", c))
            .unwrap_or(String::new())
    )
}

/*
Result is like:

```sql
SELECT sticky.*
FROM sticky
         JOIN sticky_tag ON sticky.id = sticky_tag.sticky
WHERE sticky_tag.tag IN (SELECT id as tag FROM tag WHERE (tag.namespace = 'female' AND tag.value = 'lolicon'))
GROUP BY sticky.id
HAVING COUNT(DISTINCT sticky_tag.tag) = 1
   AND sticky.id NOT IN (SELECT sticky.id
                         FROM sticky
                                  JOIN sticky_tag ON sticky.id = sticky_tag.sticky
                                  JOIN tag ON sticky_tag.tag = tag.id
                         WHERE (tag.namespace = 'misc' AND tag.value = 'nsfw'));

```
 */
fn build_sub_query_tages(tags: &Vec<ParsedTag>) -> Option<String> {
    if tags.is_empty() {
        return None;
    }
    let tags_has_all = tags.iter().filter(|tag| !tag.not).collect::<Vec<_>>();
    let tags_none_of = tags.iter().filter(|tag| tag.not).collect::<Vec<_>>();

    let mut select_sticky =
        "SELECT sticky.* FROM sticky JOIN sticky_tag ON sticky.id = sticky_tag.sticky WHERE"
            .to_string();

    let has_all_cond = if !tags_has_all.is_empty() {
        Some(format!(
            " sticky_tag.tag IN (SELECT id as tag FROM tag WHERE {}) GROUP BY sticky.id HAVING COUNT(DISTINCT sticky_tag.tag) = {}",
            tags_has_all
                .iter()
                .map(|t| {
                    format!(
                        "(tag.namespace = '{}' AND tag.value = '{}')",
                        t.namespace, t.value
                    )
                })
                .reduce(|pre, acc| format!("{} OR {}", pre, acc))
                .unwrap(),
            tags_has_all.len()
        ))
    } else {
        None
    };

    let none_of_cond = if !tags_none_of.is_empty() {
        Some(format!(
            " sticky.id NOT IN (SELECT sticky.id FROM sticky JOIN sticky_tag ON sticky.id = sticky_tag.sticky JOIN tag ON sticky_tag.tag = tag.id WHERE {})",
            tags_none_of
                .iter()
                .map(|t| {
                    format!(
                        "(tag.namespace = '{}' AND tag.value = '{}')",
                        t.namespace, t.value
                    )
                })
                .reduce(|pre, acc| format!("{} OR {}", pre, acc))
                .unwrap()
        ))
    } else {
        None
    };
    if let Some(first_cond) = has_all_cond {
        select_sticky.push_str(&first_cond);
        if let Some(second_cond) = none_of_cond {
            select_sticky.push_str(" AND");
            select_sticky.push_str(&second_cond);
        }
    } else if let Some(second_cond) = none_of_cond {
        select_sticky.push_str(&second_cond);
    }

    Some(select_sticky)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{MetaKey, ParsedTag, Search};

    #[test]
    fn test_full_serach() {
        let s = Search {
            tags: vec![
                ParsedTag {
                    value: String::from("lolicon"),
                    namespace: String::from("female"),
                    not: false,
                },
                ParsedTag {
                    not: true,
                    value: String::from("guro"),
                    namespace: String::from("female"),
                },
            ],
            keywords: vec![
                ParsedKeyword {
                    not: false,
                    value: "刷".to_string(),
                },
                ParsedKeyword {
                    not: true,
                    value: "朱重八".to_string(),
                },
            ],
            meta: vec![ParsedMeta {
                key: MetaKey::Package,
                not: false,
                value: "Inbox".to_string(),
            }],
        };
        println!("{}",build_search_sql_stmt(s, 0, 30));
    }

    #[test]
    fn test_stem() {
        assert_eq!(
            build_main_stem(
                "sticky",
                &vec![
                    ParsedKeyword {
                        not: false,
                        value: "刷".to_string()
                    },
                    ParsedKeyword {
                        not: true,
                        value: "朱重八".to_string()
                    }
                ],
                &Vec::new()
            ),
            String::from("SELECT inp.* FROM sticky as inp JOIN package on inp.package = package.id WHERE package.name = '穗穗'")
        );
        assert_eq!(
            build_main_stem(
                "sticky",
                &vec![
                ],
                &vec![
                    ParsedMeta{
                        key: MetaKey::Package,
                        not: false,
                        value: "穗穗".to_string()
                    }
                ]
            ),
            String::from("SELECT inp.* FROM sticky as inp WHERE inp.name LIKE '%刷%' AND inp.name NOT LIKE '%朱重八%'")
        );
    }

    #[test]
    fn test_tag() {
        assert_eq!(build_sub_query_tages(&vec![]), None);
        assert_eq!(
            build_sub_query_tages(&vec![
                ParsedTag {
                    value: String::from("lolicon"),
                    namespace: String::from("female"),
                    not: false,
                },
            ]),
            Some("SELECT sticky.* FROM sticky JOIN sticky_tag ON sticky.id = sticky_tag.sticky WHERE sticky_tag.tag IN (SELECT id as tag FROM tag WHERE (tag.namespace = 'female' AND tag.value = 'lolicon')) GROUP BY sticky.id HAVING COUNT(DISTINCT sticky_tag.tag) = 1".to_string())
        );
        assert_eq!(
            build_sub_query_tages(&vec![
                ParsedTag {
                    not: true,
                    value: String::from("nsfw"),
                    namespace: String::from("misc")
                }
            ]),
            Some("SELECT sticky.* FROM sticky JOIN sticky_tag ON sticky.id = sticky_tag.sticky WHERE sticky_tag.tag IN (SELECT id as tag FROM tag WHERE (tag.namespace = 'female' AND tag.value = 'lolicon')) GROUP BY sticky.id HAVING COUNT(DISTINCT sticky_tag.tag) = 1 AND sticky.id NOT IN (SELECT sticky.id FROM sticky JOIN sticky_tag ON sticky.id = sticky_tag.sticky JOIN tag ON sticky_tag.tag = tag.id WHERE (tag.namespace = 'misc' AND tag.value = 'nsfw'))".to_string())
        );
        assert_eq!(
            build_sub_query_tages(&vec![
                ParsedTag {
                    value: String::from("lolicon"),
                    namespace: String::from("female"),
                    not: false,
                },
                ParsedTag {
                    not: true,
                    value: String::from("nsfw"),
                    namespace: String::from("misc")
                }
            ]),
            Some("SELECT sticky.* FROM sticky JOIN sticky_tag ON sticky.id = sticky_tag.sticky WHERE sticky_tag.tag IN (SELECT id as tag FROM tag WHERE (tag.namespace = 'female' AND tag.value = 'lolicon')) GROUP BY sticky.id HAVING COUNT(DISTINCT sticky_tag.tag) = 1 AND sticky.id NOT IN (SELECT sticky.id FROM sticky JOIN sticky_tag ON sticky.id = sticky_tag.sticky JOIN tag ON sticky_tag.tag = tag.id WHERE (tag.namespace = 'misc' AND tag.value = 'nsfw'))".to_string())
        );
    }
}
