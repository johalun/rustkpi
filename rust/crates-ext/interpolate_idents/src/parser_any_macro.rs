// from src/libsyntax/ext/tt/macro_rules.rs

use std::cell::RefCell;
use syntax::parse::parser::Parser;
use syntax::parse::token;
use syntax::ast;
use syntax::ptr::P;

use syntax::ext::base::MacResult;
use syntax::util::small_vector::SmallVector;


macro_rules! panictry {
    ($e:expr) => ({
        use std::result::Result::{Ok, Err};
        use syntax::errors::FatalError;
        match $e {
            Ok(e) => e,
            Err(mut e) => {
                   e.emit();
                   FatalError.raise();
               }
        }
    })
}

pub struct ParserAnyMacro<'a> {
    parser: RefCell<Parser<'a>>,
}

impl<'a> ParserAnyMacro<'a> {
    pub fn new(p: Parser<'a>) -> ParserAnyMacro<'a> {
        ParserAnyMacro {
            parser: RefCell::new(p)
        }
    }
    /// Make sure we don't have any tokens left to parse, so we don't
    /// silently drop anything. `allow_semi` is so that "optional"
    /// semicolons at the end of normal expressions aren't complained
    /// about e.g. the semicolon in `macro_rules! kapow { () => {
    /// panic!(); } }` doesn't get picked up by .parse_expr(), but it's
    /// allowed to be there.
    fn ensure_complete_parse(&self, allow_semi: bool) {
        let mut parser = self.parser.borrow_mut();
        if allow_semi && parser.token == token::Semi {
            parser.bump();
        }
        if parser.token != token::Eof {
            let token_str = parser.this_token_to_string();
            let msg = format!("macro expansion ignores token `{}` and any \
                               following",
                              token_str);
            let span = parser.span;
            parser.span_err(span, &msg[..]);
        }
    }
}

impl<'a> MacResult for ParserAnyMacro<'a> {
    fn make_expr(self: Box<ParserAnyMacro<'a>>) -> Option<P<ast::Expr>> {
        let ret = panictry!(self.parser.borrow_mut().parse_expr());
        self.ensure_complete_parse(true);
        Some(ret)
    }
    fn make_pat(self: Box<ParserAnyMacro<'a>>) -> Option<P<ast::Pat>> {
        let ret = panictry!(self.parser.borrow_mut().parse_pat());
        self.ensure_complete_parse(false);
        Some(ret)
    }
    fn make_items(self: Box<ParserAnyMacro<'a>>) -> Option<SmallVector<P<ast::Item>>> {
        let mut ret = SmallVector::new();
        while let Some(item) = panictry!(self.parser.borrow_mut().parse_item()) {
            ret.push(item);
        }
        self.ensure_complete_parse(false);
        Some(ret)
    }

    fn make_impl_items(self: Box<ParserAnyMacro<'a>>)
                       -> Option<SmallVector<ast::ImplItem>> {
        let mut ret = SmallVector::new();
        loop {
            let mut parser = self.parser.borrow_mut();
            match parser.token {
                token::Eof => break,
                _ => ret.push(panictry!(parser.parse_impl_item(&mut false)))
            }
        }
        self.ensure_complete_parse(false);
        Some(ret)
    }

    fn make_stmts(self: Box<ParserAnyMacro<'a>>)
                 -> Option<SmallVector<ast::Stmt>> {
        let mut ret = SmallVector::new();
        loop {
            let mut parser = self.parser.borrow_mut();
            match parser.token {
                token::Eof => break,
                _ => match parser.parse_stmt() {
                    Ok(maybe_stmt) => match maybe_stmt {
                        Some(stmt) => ret.push(stmt),
                        None => (),
                    },
                    Err(_) => break,
                }
            }
        }
        self.ensure_complete_parse(false);
        Some(ret)
    }
}
