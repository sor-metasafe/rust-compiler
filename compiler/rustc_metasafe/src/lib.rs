#![feature(lazy_cell)]
#![feature(decl_macro)]
#![feature(panic_update_hook)]
#![feature(let_chains)]
#![recursion_limit = "256"]
#![allow(rustc::potential_query_instability)]
#![deny(rustc::untranslatable_diagnostic)]
#![deny(rustc::diagnostic_outside_of_impl)]

use std::{path::Path, fs::File, io::BufReader};

use rustc_ast::NodeId;
use rustc_data_structures::fx::{FxHashSet, FxHashMap};

pub mod hir_analysis;
pub mod ast_visitor;

use self::hir_analysis::StructRecord;

#[derive(Default)]
pub struct AnalysisRecords {
    pub structs: FxHashSet<NodeId>,
    pub struct_defs: FxHashSet<NodeId>,
    pub except_defs: FxHashSet<NodeId>
}

pub fn load_analysis(crate_name: String) -> AnalysisRecords {
    let path = Path::new("/tmp/metasafe/analysis.json");
    if path.exists() {
        let file =  File::open(path).unwrap();
        let reader = BufReader::new(file);
        let mut parsed_record: FxHashMap<String, FxHashMap<usize, StructRecord>> = serde_json::from_reader(reader).unwrap();
        let structs = parsed_record.entry(crate_name.clone()).or_default();
        let mut boxable_structs = FxHashSet::default();
        let mut except_defs = FxHashSet::default();

        structs.iter().for_each(|(_, entry)|{
            if entry.needs_box {
                boxable_structs.insert(NodeId::from_u32(entry.node_id));
            }
        });

        let mut boxable_defs = FxHashSet::default();
        parsed_record.iter().for_each(|(_,structs)|{
            structs.iter().for_each(|(_,entry)|{
                if entry.needs_box {
                    if let Some(defs) = entry.struct_defs.get(&crate_name) {
                        let collected: FxHashSet<NodeId> = defs.iter().map(|entry|{
                            NodeId::from_u32(*entry)
                        }).collect();
                        boxable_defs.extend(collected.iter())
                    }
                    if let Some(excepts) = entry.except_defs.get(&crate_name) {
                        let collected: FxHashSet<NodeId> = excepts.iter().map(|entry|{
                            NodeId::from_u32(*entry)
                        }).collect();
                        except_defs.extend(collected.iter());
                    }
                }
            });
        });

        AnalysisRecords {
            structs: boxable_structs,
            struct_defs: boxable_defs,
            except_defs
        }
    } else {
        AnalysisRecords {
            structs: FxHashSet::default(),
            struct_defs: FxHashSet::default(),
            except_defs: FxHashSet::default()
        }
    }
}
