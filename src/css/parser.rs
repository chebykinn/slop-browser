use super::selector::parse_selector;
use super::stylesheet::{Declaration, Rule, Stylesheet, Unit, Value};
use crate::render::painter::Color;
use cssparser::{Parser, ParserInput, Token};

pub fn parse_css(css: &str) -> Stylesheet {
    let mut stylesheet = Stylesheet::new();
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);

    while !parser.is_exhausted() {
        if let Ok(rule) = parse_rule(&mut parser) {
            stylesheet.add_rule(rule);
        } else {
            let _ = parser.next();
        }
    }

    stylesheet
}

fn parse_rule<'i>(parser: &mut Parser<'i, '_>) -> Result<Rule, cssparser::ParseError<'i, ()>> {
    let mut rule = Rule::new();

    let selector_str = parse_selector_string(parser)?;
    for sel_part in selector_str.split(',') {
        if let Some(selector) = parse_selector(sel_part.trim()) {
            rule.selectors.push(selector);
        }
    }

    parser.expect_curly_bracket_block()?;
    parser.parse_nested_block(|p| {
        while !p.is_exhausted() {
            if let Ok(decl) = parse_declaration(p) {
                rule.declarations.push(decl);
            } else {
                let _ = p.next();
            }
        }
        Ok(())
    })?;

    Ok(rule)
}

fn parse_selector_string<'i>(parser: &mut Parser<'i, '_>) -> Result<String, cssparser::ParseError<'i, ()>> {
    let mut selector = String::new();

    loop {
        let state = parser.state();
        let token = parser.next_including_whitespace()?;
        match token {
            Token::CurlyBracketBlock => {
                parser.reset(&state);
                break;
            }
            Token::Ident(ident) => selector.push_str(ident),
            Token::IDHash(id) => {
                selector.push('#');
                selector.push_str(id);
            }
            Token::Delim('.') => selector.push('.'),
            Token::Delim('#') => selector.push('#'),
            Token::Delim('*') => selector.push('*'),
            Token::Delim(',') => selector.push(','),
            Token::WhiteSpace(_) => selector.push(' '),
            _ => {}
        }

        if parser.is_exhausted() {
            break;
        }
    }

    Ok(selector.trim().to_string())
}

fn parse_declaration<'i>(parser: &mut Parser<'i, '_>) -> Result<Declaration, cssparser::ParseError<'i, ()>> {
    let property = parser.expect_ident()?.to_string();
    parser.expect_colon()?;

    let (value, important) = parse_value_list(parser)?;

    let _ = parser.try_parse(|p| p.expect_semicolon());

    Ok(Declaration { property, value, important })
}

/// Parse a list of values (for shorthand properties) and check for !important
fn parse_value_list<'i>(parser: &mut Parser<'i, '_>) -> Result<(Value, bool), cssparser::ParseError<'i, ()>> {
    let mut values = Vec::new();
    let mut important = false;

    loop {
        // Check for semicolon or end of block
        let state = parser.state();
        match parser.next() {
            Ok(Token::Semicolon) => {
                parser.reset(&state);
                break;
            }
            Ok(Token::CurlyBracketBlock) => {
                parser.reset(&state);
                break;
            }
            Ok(Token::Delim('!')) => {
                // Check for !important
                if let Ok(Token::Ident(ident)) = parser.next() {
                    if ident.eq_ignore_ascii_case("important") {
                        important = true;
                        break;
                    }
                }
            }
            _ => {
                parser.reset(&state);
            }
        }

        if parser.is_exhausted() {
            break;
        }

        // Try to parse a value
        match parse_single_value(parser) {
            Ok(v) => values.push(v),
            Err(_) => break,
        }

        if parser.is_exhausted() {
            break;
        }
    }

    // Return single value or list
    let value = match values.len() {
        0 => Value::Keyword(String::new()),
        1 => values.pop().unwrap(),
        _ => Value::List(values),
    };

    Ok((value, important))
}

fn parse_single_value<'i>(parser: &mut Parser<'i, '_>) -> Result<Value, cssparser::ParseError<'i, ()>> {
    let token = parser.next()?.clone();

    match token {
        Token::Ident(ident) => {
            let ident_str = ident.to_string();
            if ident_str == "auto" {
                Ok(Value::Auto)
            } else if ident_str == "none" {
                Ok(Value::None)
            } else {
                Ok(Value::Keyword(ident_str))
            }
        }
        Token::Hash(hash) | Token::IDHash(hash) => {
            if let Some(color) = Color::from_hex(&hash.to_string()) {
                Ok(Value::Color(color))
            } else {
                Ok(Value::Keyword(hash.to_string()))
            }
        }
        Token::Dimension { value, unit, .. } => {
            let unit = match unit.as_ref() {
                "px" => Unit::Px,
                "em" => Unit::Em,
                "rem" => Unit::Rem,
                "%" => Unit::Percent,
                "vh" => Unit::Vh,
                "vw" => Unit::Vw,
                _ => Unit::Px,
            };
            Ok(Value::Length(value, unit))
        }
        Token::Percentage { unit_value, .. } => {
            Ok(Value::Percentage(unit_value * 100.0))
        }
        Token::Number { value, .. } => {
            Ok(Value::Number(value))
        }
        Token::Function(name) => {
            let name = name.to_string();
            parser.parse_nested_block(|p| {
                if name == "rgb" || name == "rgba" {
                    parse_rgb_function(p, name == "rgba")
                } else {
                    Ok(Value::Keyword(name))
                }
            })
        }
        _ => Ok(Value::Keyword(String::new())),
    }
}

fn parse_rgb_function<'i>(parser: &mut Parser<'i, '_>, has_alpha: bool) -> Result<Value, cssparser::ParseError<'i, ()>> {
    let r = parse_color_component(parser)?;
    let _ = parser.try_parse(|p| p.expect_comma());

    let g = parse_color_component(parser)?;
    let _ = parser.try_parse(|p| p.expect_comma());

    let b = parse_color_component(parser)?;

    let a = if has_alpha {
        let _ = parser.try_parse(|p| p.expect_comma());
        parse_alpha_component(parser).unwrap_or(1.0)
    } else {
        1.0
    };

    Ok(Value::Color(Color::rgba(r, g, b, (a * 255.0) as u8)))
}

fn parse_color_component<'i>(parser: &mut Parser<'i, '_>) -> Result<u8, cssparser::ParseError<'i, ()>> {
    let token = parser.next()?.clone();
    match token {
        Token::Number { value, .. } => Ok(value.clamp(0.0, 255.0) as u8),
        Token::Percentage { unit_value, .. } => Ok((unit_value * 255.0).clamp(0.0, 255.0) as u8),
        _ => Ok(0),
    }
}

fn parse_alpha_component<'i>(parser: &mut Parser<'i, '_>) -> Result<f32, cssparser::ParseError<'i, ()>> {
    let token = parser.next()?.clone();
    match token {
        Token::Number { value, .. } => Ok(value.clamp(0.0, 1.0)),
        Token::Percentage { unit_value, .. } => Ok(unit_value.clamp(0.0, 1.0)),
        _ => Ok(1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule() {
        let css = "p { color: red; }";
        let stylesheet = parse_css(css);
        assert_eq!(stylesheet.rules.len(), 1);

        let rule = &stylesheet.rules[0];
        assert_eq!(rule.selectors.len(), 1);
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "color");
    }

    #[test]
    fn test_parse_hex_color() {
        let css = "div { background-color: #ff0000; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        if let Value::Color(c) = &decl.value {
            assert_eq!(c.r, 1.0);
            assert_eq!(c.g, 0.0);
            assert_eq!(c.b, 0.0);
        } else {
            panic!("Expected color value");
        }
    }

    #[test]
    fn test_parse_length() {
        let css = "div { margin: 10px; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        if let Value::Length(v, Unit::Px) = &decl.value {
            assert_eq!(*v, 10.0);
        } else {
            panic!("Expected length value");
        }
    }

    #[test]
    fn test_parse_multiple_declarations() {
        let css = "div { color: blue; font-size: 16px; margin: 10px; }";
        let stylesheet = parse_css(css);

        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].declarations.len(), 3);
        assert_eq!(stylesheet.rules[0].declarations[0].property, "color");
        assert_eq!(stylesheet.rules[0].declarations[1].property, "font-size");
        assert_eq!(stylesheet.rules[0].declarations[2].property, "margin");
    }

    #[test]
    fn test_parse_multiple_rules() {
        let css = "p { color: red; } div { color: blue; }";
        let stylesheet = parse_css(css);

        assert_eq!(stylesheet.rules.len(), 2);
    }

    #[test]
    fn test_parse_class_selector() {
        let css = ".container { width: 100px; }";
        let stylesheet = parse_css(css);

        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors.len(), 1);
    }

    #[test]
    fn test_parse_id_selector() {
        let css = "#main { height: 200px; }";
        let stylesheet = parse_css(css);

        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors.len(), 1);
    }

    #[test]
    fn test_parse_short_hex_color() {
        let css = "div { color: #f00; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        if let Value::Color(c) = &decl.value {
            assert_eq!(c.r, 1.0);
            assert_eq!(c.g, 0.0);
            assert_eq!(c.b, 0.0);
        } else {
            panic!("Expected color value");
        }
    }

    #[test]
    fn test_parse_keyword_values() {
        let css = "div { display: block; position: relative; }";
        let stylesheet = parse_css(css);

        let decl1 = &stylesheet.rules[0].declarations[0];
        if let Value::Keyword(k) = &decl1.value {
            assert_eq!(k, "block");
        } else {
            panic!("Expected keyword value");
        }
    }

    #[test]
    fn test_parse_auto_value() {
        let css = "div { margin: auto; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        assert!(matches!(decl.value, Value::Auto));
    }

    #[test]
    fn test_parse_none_value() {
        let css = "div { display: none; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        assert!(matches!(decl.value, Value::None));
    }

    #[test]
    fn test_parse_percentage() {
        let css = "div { width: 50%; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        if let Value::Percentage(v) = &decl.value {
            assert_eq!(*v, 50.0);
        } else {
            panic!("Expected percentage value");
        }
    }

    #[test]
    fn test_parse_em_unit() {
        let css = "p { font-size: 1.5em; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        if let Value::Length(v, Unit::Em) = &decl.value {
            assert_eq!(*v, 1.5);
        } else {
            panic!("Expected em length value");
        }
    }

    #[test]
    fn test_parse_rgb_function() {
        let css = "div { color: rgb(255, 128, 0); }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        if let Value::Color(c) = &decl.value {
            assert_eq!(c.r, 1.0);
            assert!((c.g - 0.502).abs() < 0.01);
            assert_eq!(c.b, 0.0);
        } else {
            panic!("Expected color value");
        }
    }

    #[test]
    fn test_parse_number_value() {
        let css = "div { z-index: 10; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        if let Value::Number(v) = &decl.value {
            assert_eq!(*v, 10.0);
        } else {
            panic!("Expected number value");
        }
    }

    #[test]
    fn test_empty_stylesheet() {
        let css = "";
        let stylesheet = parse_css(css);
        assert_eq!(stylesheet.rules.len(), 0);
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let css = "   div   {   color:   red;   }   ";
        let stylesheet = parse_css(css);
        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn test_parse_multi_value_margin() {
        let css = "div { margin: 10px 20px; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        assert_eq!(decl.property, "margin");
        assert!(matches!(decl.value, Value::List(_)));
        if let Value::List(values) = &decl.value {
            assert_eq!(values.len(), 2);
        }
    }

    #[test]
    fn test_parse_four_value_padding() {
        let css = "div { padding: 1px 2px 3px 4px; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        if let Value::List(values) = &decl.value {
            assert_eq!(values.len(), 4);
        } else {
            panic!("Expected list value");
        }
    }

    #[test]
    fn test_parse_important() {
        let css = "div { color: red !important; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        assert!(decl.important);
    }

    #[test]
    fn test_parse_not_important() {
        let css = "div { color: red; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        assert!(!decl.important);
    }

    #[test]
    fn test_parse_comments() {
        let css = "/* comment */ div { /* inline */ color: red; /* after */ }";
        let stylesheet = parse_css(css);

        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].declarations[0].property, "color");
    }

    #[test]
    fn test_parse_border_shorthand() {
        let css = "div { border: 1px solid black; }";
        let stylesheet = parse_css(css);

        let decl = &stylesheet.rules[0].declarations[0];
        assert_eq!(decl.property, "border");
        assert!(matches!(decl.value, Value::List(_)));
    }
}
