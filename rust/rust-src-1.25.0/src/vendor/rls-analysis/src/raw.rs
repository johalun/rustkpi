// Copyright 2016 The RLS Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {AnalysisLoader, Blacklist};
use listings::{DirectoryListing, ListingKind};
pub use data::{CratePreludeData, Def, DefKind, GlobalCrateId as CrateId, Import,
               Ref, Relation, RelationKind, SigElement, Signature, SpanData};
use data::Analysis;

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::time::{Instant, SystemTime};

#[derive(Debug)]
pub struct Crate {
    pub id: CrateId,
    pub analysis: Analysis,
    pub timestamp: SystemTime,
}

impl Crate {
    pub fn new(analysis: Analysis, timestamp: SystemTime) -> Crate {
        Crate {
            id: analysis.prelude.as_ref().unwrap().crate_id.clone(),
            analysis,
            timestamp
        }
    }
}

/// Reads raw analysis data for non-blacklisted crates from files in directories
/// pointed by `loader`.
pub fn read_analysis_from_files<L: AnalysisLoader>(
    loader: &L,
    crate_timestamps: HashMap<CrateId, SystemTime>,
    crate_blacklist: Blacklist,
) -> Vec<Crate> {
    let mut result = vec![];

    loader.search_directories()
    .iter()
    .inspect(|path| trace!("Considering analysis files at {}", path.display()))
    .filter_map(|p| DirectoryListing::from_path(p).ok().map(|list| (p, list)))
    .for_each(|(p, listing)| {
        let t = Instant::now();

        for l in listing.files {
            info!("Considering {:?}", l);
            if let ListingKind::File(ref time) = l.kind {
                if ignore_data(&l.name, crate_blacklist) {
                    continue;
                }

                let path = p.join(&l.name);
                // TODO: Bring back path-based timestamps, so we can discard
                // stale data before reading the file and attempting the
                // deserialization, as it can take a considerate amount of time
                // for big analysis data files.
                //let is_fresh = timestamps.get(&path).map_or(true, |t| time > t);
                read_crate_data(&path).map(|analysis| {
                    let is_fresh = {
                        let id = &analysis.prelude.as_ref().unwrap().crate_id;
                        crate_timestamps.get(id).map_or(true, |t| time > t) 
                    };
                    if is_fresh {
                        result.push(Crate::new(analysis, *time));
                    }
                });
            }
        }

        let d = t.elapsed();
        info!(
            "reading {} crates from {} in {}.{:09}s",
            result.len(),
            p.display(),
            d.as_secs(),
            d.subsec_nanos()
        );
    });

    result
}

fn ignore_data(file_name: &str, crate_blacklist: Blacklist) -> bool {
    crate_blacklist.iter()
        .any(|name| file_name.starts_with(&format!("lib{}-", name)))
}

fn read_file_contents(path: &Path) -> io::Result<String> {
    let mut file = File::open(&path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

/// Attempts to read and deserialize `Analysis` data from a JSON file at `path`,
/// returns `Some(data)` on success.
fn read_crate_data(path: &Path) -> Option<Analysis> {
    trace!("read_crate_data {:?}", path);
    let t = Instant::now();

    let buf = read_file_contents(path).or_else(|err| {
        info!("couldn't read file: {}", err);
        Err(err)
    }).ok()?;
    let s = ::rustc_serialize::json::decode(&buf).or_else(|err| {
        info!("deserialisation error: {:?}", err);
        Err(err)
    }).ok()?;

    let d = t.elapsed();
    info!(
        "reading {:?} {}.{:09}s",
        path,
        d.as_secs(),
        d.subsec_nanos()
    );

    s
}

pub fn name_space_for_def_kind(dk: DefKind) -> char {
    match dk {
        DefKind::Enum |
        DefKind::Struct |
        DefKind::Union |
        DefKind::Type |
        DefKind::ExternType |
        DefKind::Trait => 't',
        DefKind::Function |
        DefKind::Method |
        DefKind::Mod |
        DefKind::Local |
        DefKind::Static |
        DefKind::Const |
        DefKind::Tuple |
        DefKind::TupleVariant |
        DefKind::StructVariant |
        DefKind::Field => 'v',
        DefKind::Macro => 'm',
    }
}
