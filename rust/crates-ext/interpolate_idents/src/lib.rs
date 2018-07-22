#![feature(plugin_registrar, rustc_private)]

extern crate rustc;
extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::Registry;
use syntax::ast::Ident;
use syntax::tokenstream::{TokenStream, TokenTree};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult};
use syntax::parse::token::{DelimToken, Token};

mod parser_any_macro;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("interpolate_idents", interpolate_idents);
}

fn interpolate_idents<'a>(cx: &'a mut ExtCtxt,
              _: Span,
              tts: &[TokenTree]) -> Box<MacResult + 'a> {
    fn concat_idents(tts: TokenStream, delim: DelimToken) -> Option<TokenTree> {
        match delim {
            DelimToken::Bracket => {
                let mut new_ident = String::new();
                let mut new_span: Option<Span> = None;

                for token in tts.trees() {
                    match token {
                        TokenTree::Token(ref span, Token::Ident(ref ident)) => {
                            match new_span {
                                Some(ref mut s) => { *s = s.with_hi(span.hi()); },
                                None => { new_span = Some(span.clone()); },
                            }
                            new_ident.push_str(&ident.name.as_str());
                        },
                        TokenTree::Token(ref span, Token::Underscore) => {
                            match new_span {
                                Some(ref mut s) => { *s = s.with_hi(span.hi()); },
                                None => { new_span = Some(span.clone()); },
                            }
                            new_ident.push_str("_");
                        },
                        _ => return None,
                    }
                }

                match new_span {
                    Some(s) => {
                        let new_ident = Ident::from_str(&new_ident[..]);
                        Some(TokenTree::Token(s, Token::Ident(new_ident)))
                    },
                    None => None
                }
            },
            _ => None,
        }
    }

    fn map_tts(tts: TokenStream) -> TokenStream {
        // Ignore brackets preceded by a pound symbol (or a pound and an exclamation mark), so as
        // to allow attributes.
        let mut is_prev_pound = false;

        tts.trees().map(|t| {
            if is_prev_pound {
                is_prev_pound = false;
                return t.clone();
            }

            if let TokenTree::Token(_, Token::Pound) = t {
                is_prev_pound = true;
            }

            match t {
                TokenTree::Delimited(s, d) => {
                    match concat_idents(d.tts.clone().into(), d.delim) {
                        Some(t) => t,
                        None => {
                            TokenTree::Delimited(s, syntax::tokenstream::Delimited {
                                delim: d.delim,
                                tts: map_tts(d.tts.into()).into(),
                            })
                        },
                    }
                },
                TokenTree::Token(..) => t.clone(),
            }
        }).collect()
    }

    let tts: TokenStream = map_tts(tts.iter().cloned().collect());
    let tts: Vec<TokenTree> = tts.trees().collect();
    let parser = cx.new_parser_from_tts(&*tts);
    Box::new(parser_any_macro::ParserAnyMacro::new(parser)) as Box<MacResult>
}
