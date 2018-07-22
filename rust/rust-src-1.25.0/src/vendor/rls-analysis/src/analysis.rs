// Copyright 2017 The RLS Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use radix_trie::{Trie, TrieCommon};

use {Id, Span};
use raw::{CrateId, DefKind};

/// This is the main database that contains all the collected symbol information,
/// such as definitions, their mapping between spans, hierarchy and so on,
/// organized in a per-crate fashion.
#[derive(Debug)]
pub struct Analysis {
    /// Contains lowered data with global inter-crate `Id`s per each crate.
    pub per_crate: HashMap<CrateId, PerCrateAnalysis>,

    pub doc_url_base: String,
    pub src_url_base: String,
}

#[derive(Debug)]
pub struct PerCrateAnalysis {
    // Map span to id of def (either because it is the span of the def, or of
    // the def for the ref).
    pub def_id_for_span: HashMap<Span, Ref>,
    pub defs: HashMap<Id, Def>,
    pub defs_per_file: HashMap<PathBuf, Vec<Id>>,
    pub children: HashMap<Id, HashSet<Id>>,
    pub def_names: HashMap<String, Vec<Id>>,
    pub def_trie: Trie<String, Vec<Id>>,
    pub ref_spans: HashMap<Id, Vec<Span>>,
    pub globs: HashMap<Span, Glob>,
    pub impls: HashMap<Id, Vec<Span>>,

    pub root_id: Option<Id>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub enum Ref {
    // The common case - a reference to a single definition.
    Id(Id),
    // Two defs contribute to a single reference, occurs in the field name
    // shorthand, maybe other places.
    Double(Id, Id),
    // Multiple ids, we record only the first def we found, plus a count of defs.
    // Should only happen due to generated code which has not been well-filtered
    // by the compiler.
    Multi(Id, usize),
}

impl Ref {
    pub fn some_id(&self) -> Id {
        match *self {
            Ref::Id(id) => id,
            Ref::Double(id, _) => id,
            Ref::Multi(id, _) => id,
        }
    }

    pub fn add_id(&self, def_id: Id) -> Ref {
        match *self {
            Ref::Id(id) => Ref::Double(id, def_id),
            Ref::Double(id, _) => Ref::Multi(id, 3),
            Ref::Multi(id, n) => Ref::Multi(id, n + 1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Def {
    pub kind: DefKind,
    pub span: Span,
    pub name: String,
    pub qualname: String,
    pub distro_crate: bool,
    pub parent: Option<Id>,
    pub value: String,
    pub docs: String,
    // pub sig: Option<Signature>,
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub span: Span,
    pub text: String,
    pub ident_start: u32,
    pub ident_end: u32,
    pub defs: Vec<SigElement>,
    pub refs: Vec<SigElement>,
}

#[derive(Debug, Clone)]
pub struct SigElement {
    pub id: Id,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub struct Glob {
    pub value: String,
}


impl PerCrateAnalysis {
    pub fn new(timestamp: SystemTime) -> PerCrateAnalysis {
        PerCrateAnalysis {
            def_id_for_span: HashMap::new(),
            defs: HashMap::new(),
            defs_per_file: HashMap::new(),
            children: HashMap::new(),
            def_names: HashMap::new(),
            def_trie: Trie::new(),
            ref_spans: HashMap::new(),
            globs: HashMap::new(),
            impls: HashMap::new(),
            root_id: None,
            timestamp,
        }
    }
}

impl Analysis {
    pub fn new() -> Analysis {
        Analysis {
            per_crate: HashMap::new(),
            // TODO don't hardcode these
            doc_url_base: "https://doc.rust-lang.org/nightly".to_owned(),
            src_url_base: "https://github.com/rust-lang/rust/blob/master".to_owned(),
        }
    }

    pub fn timestamps(&self) -> HashMap<CrateId, SystemTime> {
        self.per_crate
            .iter()
            .map(|(id, c)| (id.clone(), c.timestamp.clone()))
            .collect()
    }

    pub fn update(&mut self, crate_id: CrateId, per_crate: PerCrateAnalysis) {
        self.per_crate.insert(crate_id, per_crate);
    }

    pub fn has_def(&self, id: Id) -> bool {
        self.per_crate.values().any(|c| c.defs.contains_key(&id))
    }

    pub fn for_each_crate<F, T>(&self, f: F) -> Option<T>
    where
        F: Fn(&PerCrateAnalysis) -> Option<T>,
    {
        for per_crate in self.per_crate.values() {
            if let Some(t) = f(per_crate) {
                return Some(t);
            }
        }

        None
    }

    pub fn for_all_crates<F, T>(&self, f: F) -> Vec<T>
    where
        F: Fn(&PerCrateAnalysis) -> Option<Vec<T>>,
    {
        let mut result = vec![];
        for per_crate in self.per_crate.values() {
            if let Some(this_crate) = f(per_crate) {
                result.extend(this_crate);
            }
        }

        result
    }

    pub fn def_id_for_span(&self, span: &Span) -> Option<Id> {
        self.ref_for_span(span).map(|r| r.some_id())
    }

    pub fn ref_for_span(&self, span: &Span) -> Option<Ref> {
        self.for_each_crate(|c| c.def_id_for_span.get(span).map(|r| r.clone()))
    }

    // Like def_id_for_span, but will only return a def_id if it is in the same
    // crate.
    pub fn local_def_id_for_span(&self, span: &Span) -> Option<Id> {
        self.for_each_crate(|c| {
            c.def_id_for_span
                .get(span)
                .map(|r| r.some_id())
                .and_then(|id| if c.defs.contains_key(&id) {
                    Some(id)
                } else {
                    None
                })
        })
    }

    pub fn with_defs<F, T>(&self, id: Id, f: F) -> Option<T>
    where
        F: Fn(&Def) -> T,
    {
        self.for_each_crate(|c| c.defs.get(&id).map(&f))
    }

    pub fn with_defs_and_then<F, T>(&self, id: Id, f: F) -> Option<T>
    where
        F: Fn(&Def) -> Option<T>,
    {
        self.for_each_crate(|c| c.defs.get(&id).and_then(&f))
    }

    pub fn with_globs<F, T>(&self, span: &Span, f: F) -> Option<T>
    where
        F: Fn(&Glob) -> T,
    {
        self.for_each_crate(|c| c.globs.get(span).map(&f))
    }

    pub fn for_each_child<F, T>(&self, id: Id, mut f: F) -> Option<Vec<T>>
    where
        F: FnMut(Id, &Def) -> T,
    {
        for per_crate in self.per_crate.values() {
            if let Some(children) = per_crate.children.get(&id) {
                return Some(
                    children
                        .iter()
                        .filter_map(|id| {
                            let def = per_crate.defs.get(id);
                            if def.is_none() {
                                info!("def not found for {}", id);
                            }
                            def.map(|def| f(*id, def))
                        })
                        .collect(),
                );
            }
        }

        Some(vec![])
    }

    pub fn with_ref_spans<F, T>(&self, id: Id, f: F) -> Option<T>
    where
        F: Fn(&Vec<Span>) -> T,
    {
        self.for_each_crate(|c| c.ref_spans.get(&id).map(&f))
    }

    pub fn with_defs_per_file<F, T>(&self, file: &Path, f: F) -> Option<T>
    where
        F: Fn(&Vec<Id>) -> T,
    {
        self.for_each_crate(|c| c.defs_per_file.get(file).map(&f))
    }

    pub fn matching_defs(&self, stem: &str) -> Vec<Def> {
        let lowered_stem = stem.to_lowercase();

        self.for_all_crates(|c| {
            c.def_trie.get_raw_descendant(&lowered_stem).map(|s| {
                s.values().flat_map(|ids| 
                    ids.iter().flat_map(|id| c.defs.get(id)).cloned()
                ).collect()
            })
        })
    }

    pub fn with_def_names<F, T>(&self, name: &str, f: F) -> Vec<T>
    where
        F: Fn(&Vec<Id>) -> Vec<T>,
    {
        self.for_all_crates(|c| c.def_names.get(name).map(&f))
    }
}
