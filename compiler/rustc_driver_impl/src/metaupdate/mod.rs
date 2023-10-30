use std::{path::{Path, PathBuf}, fs::create_dir_all, collections::hash_map::Entry};

use rustc_ast::NodeId;
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_hir::{HirId, OwnerId};
use rustc_middle::ty::{TyCtxt, Ty};
use rustc_span::sym::index;

pub mod hir_visitor;
pub mod ast_mut_visitor;
