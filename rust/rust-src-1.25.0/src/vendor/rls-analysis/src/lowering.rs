// Copyright 2016 The RLS Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! For processing the raw save-analysis data from rustc into the rls
//! in-memory representation.

use analysis::{Def, Glob, PerCrateAnalysis, Ref};
use data;
use raw::{self, RelationKind, CrateId};
use {AResult, AnalysisHost, Id, Span, NULL};
use loader::AnalysisLoader;
use util;

use span;

use std::collections::{HashSet, HashMap};
use std::collections::hash_map::Entry;
use std::iter::Extend;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::u32;

// f is a function used to record the lowered crate into analysis.
pub fn lower<F, L>(
    raw_analysis: Vec<raw::Crate>,
    base_dir: &Path,
    analysis: &AnalysisHost<L>,
    mut f: F,
) -> AResult<()>
where
    F: FnMut(&AnalysisHost<L>, PerCrateAnalysis, CrateId) -> AResult<()>,
    L: AnalysisLoader,
{
    let rss = util::get_resident().unwrap_or(0);
    let t_start = Instant::now();

    for c in raw_analysis {
        let t_start = Instant::now();

        let (per_crate, id) = CrateReader::read_crate(analysis, c, base_dir);

        let time = t_start.elapsed();
        info!(
            "Lowering {} in {:.2}s",
            format!("{} ({:?})", id.name, id.disambiguator),
            time.as_secs() as f64 + time.subsec_nanos() as f64 / 1_000_000_000.0
        );
        info!("    defs:  {}", per_crate.defs.len());
        info!("    refs:  {}", per_crate.ref_spans.len());
        info!("    globs: {}", per_crate.globs.len());

        f(analysis, per_crate, id)?;
    }

    let time = t_start.elapsed();
    let rss = util::get_resident().unwrap_or(0) as isize - rss as isize;
    info!(
        "Total lowering time: {:.2}s",
        time.as_secs() as f64 + time.subsec_nanos() as f64 / 1_000_000_000.0
    );
    info!("Diff in rss: {:.2}KB", rss as f64 / 1000.0);

    Ok(())
}

fn lower_span(raw_span: &raw::SpanData, base_dir: &Path) -> Span {
    let file_name = &raw_span.file_name;
    let file_name = if file_name.is_absolute() {
        file_name.to_owned()
    } else {
        base_dir.join(file_name)
    };

    // Rustc uses 1-indexed rows and columns, the RLS uses 0-indexed.
    span::Span::new(
        raw_span.line_start.zero_indexed(),
        raw_span.line_end.zero_indexed(),
        raw_span.column_start.zero_indexed(),
        raw_span.column_end.zero_indexed(),
        file_name,
    )
}

/// Responsible for processing the raw `data::Analysis`, including translating
/// from local crate ids to global crate ids, and creating lowered
/// `PerCrateAnalysis`.
struct CrateReader {
    /// This is effectively a map from local crate id -> global crate id, where
    /// local crate id are indices 0...external_crate_count.
    crate_map: Vec<u32>,
    base_dir: PathBuf,
    crate_name: String,
}

impl CrateReader {
    fn from_prelude(
        mut prelude: raw::CratePreludeData,
        master_crate_map: &mut HashMap<CrateId, u32>,
        base_dir: &Path,
    ) -> CrateReader {
        fn fetch_crate_index(map: &mut HashMap<CrateId, u32>,
                             id: data::GlobalCrateId) -> u32 {
            let next = map.len() as u32;
            *map.entry(id).or_insert(next)
        }
        // When reading a local crate and its external crates, we need to:
        // 1. Update a global crate id map if we encounter any new crate
        // 2. Prepare a local crate id -> global crate id map, so we can easily
        // map those when lowering symbols with local crate ids into global registry
        // It's worth noting, that we assume that local crate id is 0, whereas
        // the external crates will have num in 1..count contiguous range.
        let crate_id = prelude.crate_id;
        trace!("building crate map for {}", crate_id.name);
        let index = fetch_crate_index(master_crate_map, crate_id.clone());
        let mut crate_map = vec![index];
        trace!("  {} -> {}", crate_id.name, master_crate_map[&crate_id]);

        prelude.external_crates.sort_by(|a, b| a.num.cmp(&b.num));
        for c in prelude.external_crates {
            assert!(c.num == crate_map.len() as u32);
            let index = fetch_crate_index(master_crate_map, c.id.clone());
            crate_map.push(index);
            trace!("  {} -> {}", c.id.name, master_crate_map[&c.id]);
        }

        CrateReader {
            crate_map,
            base_dir: base_dir.to_owned(),
            crate_name: crate_id.name,
        }
    }

    /// Lowers a given `raw::Crate` into `AnalysisHost`.
    fn read_crate<L: AnalysisLoader>(
        project_analysis: &AnalysisHost<L>,
        krate: raw::Crate,
        base_dir: &Path,
    ) -> (PerCrateAnalysis, CrateId) {
        let reader = CrateReader::from_prelude(
            krate.analysis.prelude.unwrap(),
            &mut project_analysis.master_crate_map.lock().unwrap(),
            base_dir,
        );

        let mut per_crate = PerCrateAnalysis::new(krate.timestamp);

        let is_distro_crate = krate.analysis.config.distro_crate;
        reader.read_defs(krate.analysis.defs, &mut per_crate, is_distro_crate);
        reader.read_imports(krate.analysis.imports, &mut per_crate, project_analysis);
        reader.read_refs(krate.analysis.refs, &mut per_crate, project_analysis);
        reader.read_impls(krate.analysis.relations, &mut per_crate, project_analysis);

        (per_crate, krate.id)
    }

    fn read_imports<L: AnalysisLoader>(
        &self,
        imports: Vec<raw::Import>,
        analysis: &mut PerCrateAnalysis,
        project_analysis: &AnalysisHost<L>,
    ) {
        for i in imports {
            let span = lower_span(&i.span, &self.base_dir);
            if !i.value.is_empty() {
                // A glob import.
                let glob = Glob { value: i.value };
                trace!("record glob {:?} {:?}", span, glob);
                analysis.globs.insert(span, glob);
            } else if let Some(ref ref_id) = i.ref_id {
                // Import where we know the referred def.
                let def_id = self.id_from_compiler_id(ref_id);
                self.record_ref(def_id, span, analysis, project_analysis);
            }
        }
    }

    fn record_ref<L: AnalysisLoader>(
        &self,
        def_id: Id,
        span: Span,
        analysis: &mut PerCrateAnalysis,
        project_analysis: &AnalysisHost<L>,
    ) {
        if def_id != NULL && (project_analysis.has_def(def_id) || analysis.defs.contains_key(&def_id)) {
            trace!("record_ref {:?} {}", span, def_id);
            match analysis.def_id_for_span.entry(span.clone()) {
                Entry::Occupied(mut oe) => {
                    let new = oe.get().add_id(def_id);
                    oe.insert(new);
                }
                Entry::Vacant(ve) => {
                    ve.insert(Ref::Id(def_id));
                }
            }
            analysis
                .ref_spans
                .entry(def_id)
                .or_insert_with(|| vec![])
                .push(span);
        }
    }

    fn read_defs(&self, defs: Vec<raw::Def>, analysis: &mut PerCrateAnalysis, distro_crate: bool) {
        for d in defs {
            let span = lower_span(&d.span, &self.base_dir);
            let id = self.id_from_compiler_id(&d.id);
            if id != NULL && !analysis.defs.contains_key(&id) {
                let file_name = span.file.clone();
                analysis
                    .defs_per_file
                    .entry(file_name)
                    .or_insert_with(|| vec![])
                    .push(id);
                match analysis.def_id_for_span.entry(span.clone()) {
                    Entry::Occupied(_) => {
                        debug!("def already exists at span: {:?} {:?}", span, d);
                    }
                    Entry::Vacant(ve) => {
                        ve.insert(Ref::Id(id));
                    }
                }

                analysis
                    .def_names
                    .entry(d.name.clone())
                    .or_insert_with(|| vec![])
                    .push(id);

                // NOTE not every Def will have a name, e.g. test_data/hello/src/main is analyzed with an implicit module
                // that's fine, but no need to index in def_trie
                if d.name != "" {
                    analysis.def_trie.map_with_default(d.name.to_lowercase(), |v| v.push(id), vec![id]);
                }
                
                let parent = d.parent.map(|id| self.id_from_compiler_id(&id));
                if let Some(parent) = parent {
                    let children = analysis
                        .children
                        .entry(parent)
                        .or_insert_with(HashSet::new);
                    children.insert(id);
                }
                if !d.children.is_empty() {
                    let children_for_id = analysis.children.entry(id).or_insert_with(HashSet::new);
                    children_for_id
                        .extend(d.children.iter().map(|id| self.id_from_compiler_id(id)));
                }

                let def = Def {
                    kind: d.kind,
                    span: span,
                    name: d.name,
                    value: d.value,
                    qualname: format!("{}{}", self.crate_name, d.qualname),
                    distro_crate,
                    parent: parent,
                    docs: d.docs,
                    // sig: d.sig.map(|ref s| self.lower_sig(s, &self.base_dir)),
                };
                trace!(
                    "record def: {:?}/{:?} ({}): {:?}",
                    id,
                    d.id,
                    self.crate_map[d.id.krate as usize],
                    def
                );

                if d.kind == super::raw::DefKind::Mod && def.name == "" {
                    assert!(analysis.root_id.is_none());
                    analysis.root_id = Some(id);
                }

                analysis.defs.insert(id, def);
            }
        }

        // We must now run a pass over the defs setting parents, because
        // save-analysis often omits parent info.
        for (parent, children) in &analysis.children {
            for c in children {
                analysis
                    .defs
                    .get_mut(c)
                    .map(|def| def.parent = Some(*parent));
            }
        }
    }

    fn read_refs<L: AnalysisLoader>(
        &self,
        refs: Vec<raw::Ref>,
        analysis: &mut PerCrateAnalysis,
        project_analysis: &AnalysisHost<L>,
    ) {
        for r in refs {
            let def_id = self.id_from_compiler_id(&r.ref_id);
            let span = lower_span(&r.span, &self.base_dir);
            self.record_ref(def_id, span, analysis, project_analysis);
        }
    }

    fn read_impls<L: AnalysisLoader>(
        &self,
        relations: Vec<raw::Relation>,
        analysis: &mut PerCrateAnalysis,
        project_analysis: &AnalysisHost<L>,
    ) {
        for r in relations {
            if r.kind != RelationKind::Impl {
                continue;
            }
            let self_id = self.id_from_compiler_id(&r.from);
            let trait_id = self.id_from_compiler_id(&r.to);
            let span = lower_span(&r.span, &self.base_dir);
            if self_id != NULL {
                if let Some(self_id) = abs_ref_id(self_id, analysis, project_analysis) {
                    trace!("record impl for self type {:?} {}", span, self_id);
                    analysis
                        .impls
                        .entry(self_id)
                        .or_insert_with(|| vec![])
                        .push(span.clone());
                }
            }
            if trait_id != NULL {
                if let Some(trait_id) = abs_ref_id(trait_id, analysis, project_analysis) {
                    trace!("record impl for trait {:?} {}", span, trait_id);
                    analysis
                        .impls
                        .entry(trait_id)
                        .or_insert_with(|| vec![])
                        .push(span);
                }
            }
        }
    }

    // fn lower_sig(&self, raw_sig: &raw::Signature, base_dir: &Path) -> Signature {
    //     Signature {
    //         span: lower_span(&raw_sig.span, base_dir),
    //         text: raw_sig.text.clone(),
    //         ident_start: raw_sig.ident_start as u32,
    //         ident_end: raw_sig.ident_end as u32,
    //         defs: raw_sig.defs.iter().map(|se| self.lower_sig_element(se)).collect(),
    //         refs: raw_sig.refs.iter().map(|se| self.lower_sig_element(se)).collect(),
    //     }
    // }

    // fn lower_sig_element(&self, raw_se: &raw::SigElement) -> SigElement {
    //     SigElement {
    //         id: self.id_from_compiler_id(&raw_se.id),
    //         start: raw_se.start,
    //         end: raw_se.end,
    //     }
    // }

    /// Recreates resulting crate-local (`u32`, `u32`) id from compiler
    /// to a global `u64` `Id`, mapping from a local to global crate id.
    fn id_from_compiler_id(&self, id: &data::Id) -> Id {
        if id.krate == u32::MAX || id.index == u32::MAX {
            return NULL;
        }

        let krate = self.crate_map[id.krate as usize] as u64;
        // Use global crate number for high order bits,
        // then index for least significant bits.
        Id((krate << 32) | (id.index as u64))
    }
}

fn abs_ref_id<L: AnalysisLoader>(
    id: Id,
    analysis: &PerCrateAnalysis,
    project_analysis: &AnalysisHost<L>,
) -> Option<Id> {
    if project_analysis.has_def(id) || analysis.defs.contains_key(&id) {
        return Some(id);
    }

    // TODO
    None
}
