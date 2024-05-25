use nom::branch::alt;
use nom::bytes::complete::{escaped, tag};
use nom::character::complete::{none_of, one_of, space1};
use nom::combinator::{all_consuming, map, map_res, opt, recognize};
use nom::multi::{many1, separated_list0};
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;

pub mod sql;

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct ParsedTag {
    pub not: bool,
    pub namespace: String,
    pub value: String,
}

impl ToString for ParsedTag {
    fn to_string(&self) -> String {
        format!(
            "{}\"{}\":\"{}\"",
            if self.not { "-" } else { "" },
            self.namespace.replace("\"", "\\\""),
            self.value.replace("\"", "\\\"")
        )
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub enum MetaKey {
    DateStart,
    DateEnd,
    Package,
    Ty,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ParsedMeta {
    not: bool,
    key: MetaKey,
    value: String,
}

impl ToString for ParsedMeta {
    fn to_string(&self) -> String {
        format!(
            "{}{}:\"{}\"",
            if self.not { "-" } else { "" },
            self.key.to_string(),
            self.value.replace("\"", "\\\"")
        )
    }
}

impl MetaKey {
    fn from(key: &str) -> Result<MetaKey, String> {
        match key {
            "date-start" | "DateStart" => Ok(MetaKey::DateStart),
            "date-end" | "DateEnd" => Ok(MetaKey::DateEnd),
            "package" | "pkg" => Ok(MetaKey::Package),
            _ => Err(format!("Unrecongized key: {}", key)),
        }
    }
}
impl ToString for MetaKey {
    fn to_string(&self) -> String {
        match self {
            MetaKey::DateStart => String::from("date-start"),
            MetaKey::DateEnd => String::from("date-end"),
            MetaKey::Package => String::from("package"),
            MetaKey::Ty => String::from("type"),
        }
    }
}
#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct ParsedKeyword {
    not: bool,
    value: String,
}

impl ToString for ParsedKeyword {
    fn to_string(&self) -> String {
        format!(
            "{}\"{}\"",
            if self.not { "-" } else { "" },
            self.value.replace("\"", "\\\"")
        )
    }
}

#[derive(Debug, PartialEq)]
enum ParsedItem {
    Tag(ParsedTag),
    Kwd(ParsedKeyword),
    Meta(ParsedMeta),
}

impl ToString for ParsedItem {
    fn to_string(&self) -> String {
        match self {
            ParsedItem::Tag(v) => v.to_string(),
            ParsedItem::Kwd(v) => v.to_string(),
            ParsedItem::Meta(v) => v.to_string(),
        }
    }
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    recognize(many1(none_of(" :\"\t\r\n")))(input)
}

fn parse_str(input: &str) -> IResult<&str, &str> {
    return delimited(
        tag("\""),
        escaped(none_of("\"\\"), '\\', one_of(r#""b"#)),
        tag("\""),
    )(input);
}

fn parse_block(input: &str) -> IResult<&str, &str> {
    alt((parse_identifier, parse_str))(input)
}

fn parse_sticky_tag(input: &str) -> IResult<&str, ParsedTag> {
    map(
        tuple((opt(tag("-")), parse_block, tag(":"), parse_block)),
        |(prefix, namespace, _, value)| ParsedTag {
            not: prefix.map(|_| true).unwrap_or(false),
            namespace: namespace.to_owned(),
            value: value.to_owned(),
        },
    )(input)
}

fn parse_meta_tag(input: &str) -> IResult<&str, ParsedMeta> {
    map_res(
        tuple((
            opt(tag("-")),
            preceded(tag("$"), parse_block),
            tag(":"),
            parse_block,
        )),
        |(prefix, key, _, value)| {
            MetaKey::from(key).map(|key| ParsedMeta {
                not: prefix.map(|_| true).unwrap_or(false),
                key,
                value: value.to_string(),
            })
        },
    )(input)
}

fn parse_keyward(input: &str) -> IResult<&str, ParsedKeyword> {
    map(tuple((opt(tag("-")), parse_block)), |(prefix, v)| {
        ParsedKeyword {
            not: prefix.map(|_| true).unwrap_or(false),
            value: v.to_owned(),
        }
    })(input)
}

fn parse_search_item(input: &str) -> IResult<&str, ParsedItem> {
    alt((
        map(parse_meta_tag, |t| ParsedItem::Meta(t)),
        map(parse_sticky_tag, |t| ParsedItem::Tag(t)),
        map(parse_keyward, |t| ParsedItem::Kwd(t)),
    ))(input)
}

fn parse_search_list(input: &str) -> IResult<&str, Vec<ParsedItem>> {
    all_consuming(separated_list0(many1(space1), parse_search_item))(input.trim())
}

pub fn parse_serach(input: &str) -> Result<Search, String> {
    let items = parse_search_list(input).map_err(|e| e.to_string())?.1;
    let mut tags = Vec::new();
    let mut meta = Vec::new();
    let mut keywords = Vec::new();
    for i in items {
        match i {
            ParsedItem::Tag(t) => tags.push(t),
            ParsedItem::Kwd(k) => keywords.push(k),
            ParsedItem::Meta(m) => meta.push(m),
        }
    }
    Ok(Search {
        tags,
        keywords,
        meta,
    })
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Search {
    tags: Vec<ParsedTag>,
    keywords: Vec<ParsedKeyword>,
    meta: Vec<ParsedMeta>,
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list() {
        assert_eq!(
            parse_search_list("female:lolicon arden character:\"花丸 リリィ\" -misc:nsfw").unwrap(),
            (
                "",
                vec![
                    ParsedItem::Tag(ParsedTag {
                        not: false,
                        namespace: "female".to_string(),
                        value: "lolicon".to_string()
                    }),
                    ParsedItem::Kwd(ParsedKeyword {
                        not: false,
                        value: "arden".to_string()
                    }),
                    ParsedItem::Tag(ParsedTag {
                        not: false,
                        namespace: "character".to_string(),
                        value: "花丸 リリィ".to_string()
                    }),
                    ParsedItem::Tag(ParsedTag {
                        not: true,
                        namespace: "misc".to_string(),
                        value: "nsfw".to_string()
                    }),
                ]
            )
        );
        assert_eq!(
            parse_search_list(
                "author:nachoneko $package:\"NachoNeko Stickies\" -sleep -\"夜食 配信\""
            )
            .unwrap(),
            (
                "",
                vec![
                    ParsedItem::Tag(ParsedTag {
                        not: false,
                        namespace: "author".to_string(),
                        value: "nachoneko".to_string()
                    }),
                    ParsedItem::Meta(ParsedMeta {
                        not: false,
                        key: MetaKey::Package,
                        value: "NachoNeko Stickies".to_string()
                    }),
                    ParsedItem::Kwd(ParsedKeyword {
                        not: true,
                        value: "sleep".to_string()
                    }),
                    ParsedItem::Kwd(ParsedKeyword {
                        not: true,
                        value: "夜食 配信".to_string()
                    })
                ]
            )
        );
    }

    #[test]
    fn test_parse_item() {
        assert_eq!(
            parse_search_list("SakuradaShiro").unwrap(),
            (
                "",
                vec![ParsedItem::Kwd(ParsedKeyword {
                    not: false,
                    value: "SakuradaShiro".to_string()
                })]
            )
        );
        assert_eq!(
            parse_search_list("狐と夏風").unwrap(),
            (
                "",
                vec![ParsedItem::Kwd(ParsedKeyword {
                    not: false,
                    value: "狐と夏風".to_string()
                })]
            )
        );
        assert_eq!(
            parse_search_list("female:kemonomimi").unwrap(),
            (
                "",
                vec![ParsedItem::Tag(ParsedTag {
                    not: false,
                    namespace: "female".to_string(),
                    value: "kemonomimi".to_string()
                })]
            )
        );
        assert_eq!(
            parse_search_list(r#""花丸 リリィ""#).unwrap(),
            (
                "",
                vec![ParsedItem::Kwd(ParsedKeyword {
                    not: false,
                    value: "花丸 リリィ".to_string()
                })]
            )
        );
    }
}
