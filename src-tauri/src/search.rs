use std::fmt::Display;

#[derive(Debug)]
enum SearchStmt<'a> {
    Keyowrd(&'a str),
    Tag(&'a str, &'a str),
}

type Loc = (usize, usize);

#[derive(Debug)]
enum ErrorKind {
    IncompleteString,
    IncompleteTag,
}

#[derive(Debug)]
pub struct SearchError {
    kind: ErrorKind,
    stmt: String,
    loc: Loc,
}

impl Display for SearchError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({}:{}): {}", self.kind, self.loc.0, self.loc.1, self.stmt)
    }
}

impl std::error::Error for SearchError {

}

fn lexer<'a>(stmt: &'a str) -> Result<Vec<SearchStmt<'a>>, SearchError> {
    let mut chars = stmt.trim().char_indices();
    let mut stack = Vec::new();
    let mut references = Vec::new();
    let mut begin = chars.next();
    while let Some((begin_pos, begin_ch)) = begin {
        match begin_ch {
            ' ' | '\t' => begin = chars.next(),
            ':' => {
                begin = chars.next();
                stack.push(":");
            },
            '"' => {
                let mut end = chars.next();
                while let Some((_, end_ch)) = end {
                    if end_ch != '"' {
                        end = chars.next();
                    }else{
                        break;
                    }
                }
                if let Some((end_pos, '"')) = end {
                    stack.push(stmt.get(begin_pos + 1..end_pos).unwrap());
                    begin = chars.next();
                } else {
                    return Err(SearchError {
                        kind: ErrorKind::IncompleteString,
                        stmt: stmt.to_owned(),
                        loc: (begin_pos, references.len()),
                    });
                }
            }
            _ => {
                let mut end = chars.next();
                while let Some((_, end_ch)) = end {
                    if end_ch == ':' || end_ch == ' ' || end_ch == '\t' || end_ch == '"' {
                        begin = end;
                        break;
                    }
                    end = chars.next();
                }
                if end.is_none() {
                    begin = None;
                }
                let end_pos = end.map_or(stmt.len(), |x| x.0);
                stack.push(stmt.get(begin_pos..end_pos).unwrap());
            }
        }
        if stack.len() >= 3 {
            if stack.get(stack.len() - 2).unwrap() == &":" {
                let value = stack.pop().unwrap();
                stack.pop().unwrap();
                let namespace = stack.pop().unwrap();
                references.push(SearchStmt::Tag(namespace, value));
            }
        }
    }
    while !stack.is_empty() {
        let tok = stack.pop().unwrap().trim();
        if tok == ":" {
            return Err(SearchError{ kind: ErrorKind::IncompleteTag, stmt: stmt.to_owned(), loc: (0, 0) });
        }
        references.push(SearchStmt::Keyowrd(tok));
    }

    Ok(references)
}

pub fn build_search_sql(search_stmt: &str) -> Result<String, SearchError> {
    let stmts = lexer(search_stmt)?;
    let mut tag_select = Vec::new();
    let mut kwd_where = Vec::new();

    for stmt in stmts{
        match stmt{
            SearchStmt::Keyowrd(kwd) => kwd_where.push(format!("summary LIKE '%{}%' OR desc LIKE '%{}%'", kwd, kwd)),
            SearchStmt::Tag(namespace, value) => tag_select.push(format!("SELECT meme_id FROM tag_table WHERE namespace = '{}' AND value LIKE '{}%'", namespace, value)),
        }
    }
    let from_table = if tag_select.is_empty() {
        "meme".to_owned()
    } else {
        format!(
            "(
                WITH tag_table AS (
                    SELECT * FROM meme_tag LEFT JOIN tag ON meme_tag.tag_id = tag.id
                )
                {}
            ) LEFT JOIN meme ON meme_id = meme.id",
            tag_select.into_iter().reduce(|acc, cur| 
                format!("{}\nINTERSECT\n{}", acc, cur)
            ).unwrap()
        )
    };
    let where_stmt = if kwd_where.is_empty() {
        "".to_owned()
    } else {
        let mut wd = String::from("WHERE ");
        wd.push_str(
    &kwd_where.into_iter().reduce(|acc, pre| 
                format!("{} OR {}", acc, pre)
            ).unwrap()
        );
        wd
    };

    let result = format!("SELECT * FROM {} {}", from_table, where_stmt);



    Ok(result)
}


mod tests {

    #[test]
    fn test_lexer() {
        println!("{:?}", crate::search::lexer("group character:\"aoi sora\""));
    }
}
