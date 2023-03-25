use miette::{miette, Diagnostic, Result, SourceSpan, WrapErr};
use thiserror::Error;

pub struct Template {
    parts: Vec<TemplatePart>,
}

impl Template {
    pub fn parse<S>(template: S) -> Result<Template>
    where
        S: AsRef<str>,
    {
        TemplateParser::new(template.as_ref())
            .parse()
            .map(|parts| Template { parts })
            .wrap_err("Could not parse template")
    }

    pub fn resolve<'a, F>(&self, resolver: F) -> Result<String>
    where
        F: Fn(&str) -> Option<&'a str>,
    {
        let mut result = String::new();

        for part in &self.parts {
            let resolved = match part {
                TemplatePart::Text(text) => text.as_str(),
                TemplatePart::Variable(variable) => {
                    resolver(variable.as_str()).ok_or_else(|| {
                        miette!("Could not resolve variable '{}' in template", variable)
                    })?
                }
            };
            result += resolved;
        }

        Ok(result)
    }
}

enum TemplatePart {
    Text(String),
    Variable(String),
}

type ParseResult<T> = Result<T, ParseError>;

struct TemplateParser {
    input: Vec<char>,
    cursor: usize,
    byte_offset: usize,
}

impl TemplateParser {
    fn new(input: &str) -> TemplateParser {
        TemplateParser {
            input: input.chars().collect(),
            cursor: 0,
            byte_offset: 0,
        }
    }

    fn has_next(&self) -> bool {
        self.cursor < self.input.len()
    }

    fn parse(&mut self) -> ParseResult<Vec<TemplatePart>> {
        let mut result: Vec<TemplatePart> = Vec::new();
        let mut buffer = String::new();

        while self.has_next() {
            let c = self.next()?;
            match c {
                '\\' => {
                    buffer += self.parse_escape()?.as_str();
                }
                '$' => {
                    if !buffer.is_empty() {
                        result.push(TemplatePart::Text(buffer.clone()));
                        buffer.clear();
                    }

                    result.push(self.parse_variable()?);
                }
                _ => buffer.push(c),
            }
        }

        if !buffer.is_empty() {
            result.push(TemplatePart::Text(buffer));
        }

        Ok(result)
    }

    fn is_valid_variable_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '.'
    }

    fn parse_error<M>(&self, msg: M, start_offset: usize) -> ParseError
    where
        M: AsRef<str>,
    {
        ParseError {
            msg: msg.as_ref().to_string(),
            src: self.input.iter().collect(),
            span: self.span_from(start_offset),
            cause: None,
        }
    }

    fn span_from(&self, start_offset: usize) -> SourceSpan {
        (start_offset, self.byte_offset - start_offset).into()
    }

    fn peek(&self) -> ParseResult<char> {
        self.input
            .get(self.cursor)
            .copied()
            .ok_or_else(|| self.parse_error("Cannot peek: reached end of text", self.byte_offset))
    }

    fn next(&mut self) -> ParseResult<char> {
        let c = self.input.get(self.cursor).copied().ok_or_else(|| {
            self.parse_error("Cannot read: reached end of text", self.byte_offset)
        })?;
        self.cursor += 1;
        self.byte_offset += c.len_utf8();
        Ok(c)
    }

    fn parse_escape(&mut self) -> ParseResult<String> {
        let c = self
            .next()
            .map_err(|e| e.and_then("Could not read escape"))?;
        let result: String = match c {
            '\\' => '\\'.into(),
            '$' => '$'.into(),
            _ => format!("\\{}", c),
        };
        Ok(result)
    }

    fn parse_variable(&mut self) -> ParseResult<TemplatePart> {
        let start = self.byte_offset;
        let next = self.next()?;

        if next == '{' {
            self.parse_variable_in_brackets(start)
        } else if Self::is_valid_variable_char(next) {
            let mut var_name = String::from(next);
            self.parse_variable_name(&mut var_name)?;
            Ok(TemplatePart::Variable(var_name))
        } else {
            let msg = format!(
                "Expected variable name or curly brackets after $, found {}",
                next
            );
            Err(self.parse_error(msg, start))
        }
    }

    fn parse_variable_name(&mut self, out: &mut String) -> ParseResult<()> {
        while let Ok(c) = self.peek() {
            if !Self::is_valid_variable_char(c) {
                break;
            }

            out.push(c);
            self.next()?;
        }

        Ok(())
    }

    fn parse_variable_in_brackets(&mut self, start: usize) -> ParseResult<TemplatePart> {
        while self.peek()?.is_whitespace() {
            self.next()?; // consume all leading whitespace
        }

        let var_name_start = self.byte_offset;
        let mut var_name = String::new();
        self.parse_variable_name(&mut var_name)?;

        if var_name.is_empty() {
            return Err(self.parse_error("No variable name found inside brackets", var_name_start));
        }

        while self.has_next() && self.peek()?.is_whitespace() {
            self.next()?; // consume all trailing whitespace
        }

        match self.next() {
            Ok('}') => Ok(TemplatePart::Variable(var_name)),
            Ok(_) => Err(self.parse_error("Unclosed brackets", start)),
            Err(err) => Err(err
                .and_then("Unclosed brackets")
                .with_span(self.span_from(start))),
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("{msg}")]
struct ParseError {
    msg: String,
    #[source_code]
    src: String,
    #[label("here")]
    span: SourceSpan,
    #[source]
    cause: Option<Box<ParseError>>,
}

impl ParseError {
    fn and_then<M>(self, msg: M) -> Self
    where
        M: AsRef<str>,
    {
        Self {
            msg: msg.as_ref().to_string(),
            src: self.src.clone(),
            span: self.span,
            cause: Some(Box::new(self)),
        }
    }

    fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = span;
        self
    }
}
