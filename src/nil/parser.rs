use pest::{
    Parser,
    error::Error,
    iterators::{
        Pair,
        Pairs,
    },
};

use crate::common::UWord;

// TODO: Refactor this mod

fn parse_int(s: &str) -> UWord {
    fn replace_underscore(s: &str) -> std::borrow::Cow<str> {
        if s.contains("_") {
            let s: String = s
                .chars()
                .filter(|&c| c != '_')
                .collect();

            s.into()
        } else {
            s.into()
        }
    }

    let (s, rad) = if s.starts_with("0b") {
        let s = s.trim_start_matches("0b");
        (replace_underscore(s), 2)
    } else if s.starts_with("0o") {
        let s = s.trim_start_matches("0o");
        (replace_underscore(s), 8)
    } else if s.starts_with("0x") {
        let s = s.trim_start_matches("0x");
        (replace_underscore(s), 16)
    } else {
        (s.into(), 10)
    };

    UWord::from_str_radix(&*s, rad).unwrap()
}

#[derive(Copy, Clone, Debug)]
enum ConstExpr<'s> {
    Int(UWord),
    Name(&'s str),
    Ternary,
    Cmp(&'s str),
    And,
    Or,
    Not,
    Add(&'s str),
    Mul(&'s str),
    Operator(&'s str),
}

fn exec_const_expr(expr: &[ConstExpr]) -> Option<UWord> {
    if expr.is_empty() {
        return None;
    }

    let mut stack = Vec::with_capacity(8);

    for &ex in expr {
        let res = match ex {
            ConstExpr::Int(int) => int,
            ConstExpr::Name(_) => 0,
            ConstExpr::Ternary => {
                let e = stack.pop()?;
                let t = stack.pop()?;
                let i = stack.pop()?;

                if i != 0 { t } else { e }
            }
            ConstExpr::Cmp(cmp) => {
                let r = stack.pop()?;
                let l = stack.pop()?;

                let res = match cmp {
                    "==" => l == r,
                    "!=" => l != r,
                    "<=" => l <= r,
                    "<" => l < r,
                    ">=" => l >= r,
                    ">" => l > r,
                    _ => unreachable!()
                };

                if res { 1 } else { 0 }
            }
            ConstExpr::And => {
                let r = stack.pop()?;
                let l = stack.pop()?;

                if l != 0 && r != 0 { 1 } else { 0 }
            }
            ConstExpr::Or => {
                let r = stack.pop()?;
                let l = stack.pop()?;

                if l != 0 || r != 0 { 1 } else { 0 }
            }
            ConstExpr::Not => {
                let a = stack.pop()?;

                if a != 0 { 0 } else { 1 }
            }
            ConstExpr::Add(add) => {
                let r = stack.pop()?;
                let l = stack.pop()?;

                match add {
                    "+" => l.wrapping_add(r),
                    "-" => l.wrapping_sub(r),
                    _ => unreachable!()
                }
            }
            ConstExpr::Mul(mul) => {
                let r = stack.pop()?;
                let l = stack.pop()?;

                match mul {
                    "*" => l.wrapping_mul(r),
                    "/" => l.wrapping_div(r),
                    "%" => l.wrapping_rem(r),
                    _ => unreachable!()
                }
            }
            ConstExpr::Operator(op) => {
                match op {
                    "len" => 0,
                    "size" => 0,
                    "align" => 0,
                    _ => unreachable!()
                }
            }
        };

        stack.push(res);
    }

    let res = stack.pop();
    assert!(stack.is_empty());
    res
}

#[derive(Parser)]
#[grammar = "./nil/syntax.pest"]
pub struct NilParser;

fn parse_binary<'r, F>(mut pairs: Pairs<'r, Rule>, exprs: &mut Vec<ConstExpr<'r>>, mut get_expr: F)
                       -> Result<(), Error<Rule>>
    where
        F: FnMut(&'r str) -> ConstExpr<'r>,
{
    let left = pairs.next().unwrap();
    parse_const_expr(left, exprs).unwrap();

    while let Some(op) = pairs.next() {
        let right = pairs.next().unwrap();
        parse_const_expr(right, exprs).unwrap();

        let expr = get_expr(op.as_str());
        exprs.push(expr)
    }

    Ok(())
}

fn parse_const_expr<'r>(pair: Pair<'r, Rule>, exprs: &mut Vec<ConstExpr<'r>>)
                        -> Result<(), Error<Rule>> {
    match pair.as_rule() {
        Rule::nil => {
            let mut inner = pair.into_inner();
            let const_expr = inner.next().unwrap();
            parse_const_expr(const_expr, exprs).unwrap();

            let _eof = inner.next().unwrap();
        }
        Rule::const_expr => {
            let mut inner = pair.into_inner();
            let next = inner.next().unwrap();
            parse_const_expr(next, exprs).unwrap();
        }
        Rule::ternary => {
            let mut inner = pair.into_inner();
            let _kw_if = inner.next().unwrap();

            let const_expr = {
                let next = inner.next().unwrap();

                if next.as_rule() == Rule::nl_comment {
                    inner.next().unwrap()
                } else {
                    next
                }
            };
            parse_const_expr(const_expr, exprs).unwrap();

            let _kw_then = {
                let next = inner.next().unwrap();

                if next.as_rule() == Rule::nl_comment {
                    inner.next().unwrap()
                } else {
                    next
                }
            };

            let const_expr = {
                let next = inner.next().unwrap();

                if next.as_rule() == Rule::nl_comment {
                    inner.next().unwrap()
                } else {
                    next
                }
            };
            parse_const_expr(const_expr, exprs).unwrap();

            let _kw_else = {
                let next = inner.next().unwrap();

                if next.as_rule() == Rule::nl_comment {
                    inner.next().unwrap()
                } else {
                    next
                }
            };

            let const_expr = {
                let next = inner.next().unwrap();

                if next.as_rule() == Rule::nl_comment {
                    inner.next().unwrap()
                } else {
                    next
                }
            };

            parse_const_expr(const_expr, exprs).unwrap();

            exprs.push(ConstExpr::Ternary);
        }
        Rule::cmp => parse_binary(pair.into_inner(), exprs, ConstExpr::Cmp)?,
        Rule::and => parse_binary(pair.into_inner(), exprs, |_| ConstExpr::And)?,
        Rule::or => parse_binary(pair.into_inner(), exprs, |_| ConstExpr::Or)?,
        Rule::not => {
            let mut inner = pair.into_inner();

            let mut ns: usize = 0;
            let not = loop {
                let rule = inner.next().unwrap();
                if let Rule::kw_not = rule.as_rule() {
                    ns += 1;
                    continue;
                } else {
                    break rule;
                }
            };

            parse_const_expr(not, exprs).unwrap();

            for _ in 0..ns { exprs.push(ConstExpr::Not) }
        }
        Rule::add => parse_binary(pair.into_inner(), exprs, ConstExpr::Add)?,
        Rule::mul => parse_binary(pair.into_inner(), exprs, ConstExpr::Mul)?,
        Rule::operator => {
            let mut inner = pair.into_inner();
            let operator_name = inner.next().unwrap();
            let _operator_arg = inner.next().unwrap();

            exprs.push(ConstExpr::Operator(operator_name.as_str()))
        }
        Rule::int => {
            let int = parse_int(pair.as_str());
            exprs.push(ConstExpr::Int(int))
        }
        Rule::ident => {
            let ident = pair.as_str();

            match ident {
                "if" | "then" | "else" => panic!("Keyword!"),
                _ => (),
            }

            exprs.push(ConstExpr::Name(ident))
        }
        rule => {
            println!("{:?}", exprs);
            println!("{:?}", rule);
            println!("{:?}", pair);
            unreachable!()
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let code = r#"if 0 or 0 or 1or 0  // todo //
        then
            if  kek  ==  /***/ 0xFF34
            then  not not ( len(else) /***/ * x)
            else 12+(1 and IF)*2 +size ( _as4 ) // lol
        else 2*  2"#;

        let nil = NilParser::parse(Rule::nil, code)
            .unwrap()
            .next()
            .unwrap();

        let mut exprs = Vec::new();
        let _ = parse_const_expr(nil, &mut exprs).unwrap();

        let res = exec_const_expr(exprs.as_slice()).unwrap();
        assert_eq!(res, 12);
    }
}
