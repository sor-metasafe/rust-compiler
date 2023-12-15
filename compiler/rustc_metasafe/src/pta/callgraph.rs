use std::{path::Path, rc::Rc};

use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_middle::{
    mir::{mono::MonoItem, TerminatorKind},
    ty::{self, TyCtxt},
};
use rustc_span::def_id::{DefId, DefIdSet};

pub struct CallSite {
    //id: usize,
    caller: DefId,
    callee: DefId,
    //args: Vec<Operand<'tcx>>,
    //destination: Place<'tcx>,
}

impl CallSite {
    fn new(
        //id: usize,
        caller: DefId,
        callee: DefId,
        //args: Vec<Operand<'tcx>>,
        //destination: Place<'tcx>,
    ) -> Self {
        //Self { id, caller, callee, args, destination }
        Self { caller, callee }
    }

    fn caller(&self) -> DefId {
        self.caller
    }

    fn callee(&self) -> DefId {
        self.callee
    }

    /*fn args(&self) -> &[Operand<'tcx>] {
        &self.args
    }

    fn id(&self) -> usize {
        self.id
    }*/
}

pub struct CallGraph<'tcx> {
    tcx: TyCtxt<'tcx>,
    // The next callsite-ID to assign a callsite
    next_callsite_id: usize,
    // maps the callsite-ID to the actual callsite in this graph
    callsites: FxHashMap<usize, Rc<CallSite>>,
    // maps a given function to its callsites, here we can find all callers
    function2callsite: FxHashMap<DefId, FxHashSet<usize>>, // maps a given function to the callees?
}

impl<'tcx> CallGraph<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self {
            tcx,
            next_callsite_id: 0,
            callsites: FxHashMap::default(),
            function2callsite: FxHashMap::default(),
        }
    }

    fn next_callsite_id(&mut self) -> usize {
        let id = self.next_callsite_id;
        self.next_callsite_id += 1;
        id
    }

    pub fn build(tcx: TyCtxt<'tcx>) -> Self {
        let mut call_graph = Self::new(tcx);
        if let Some((main_did, _)) = tcx.entry_fn(()) {
            if main_did.is_local() {
                let (_items, cgus) = tcx.collect_and_partition_mono_items(());
                let mut visited = DefIdSet::default();

                for cgu in cgus {
                    for item in cgu.items().keys() {
                        if let MonoItem::Fn(ref instance) = item {
                            let did = instance.def_id();
                            if !visited.insert(did) {
                                continue;
                            }
                            let body = tcx.instance_mir(instance.def);
                            for block in body.basic_blocks.iter() {
                                if let Some(terminator) = &block.terminator {
                                    if let TerminatorKind::Call {
                                        func, args: _, destination: _, ..
                                    } = &terminator.kind
                                    {
                                        let callee_ty = func.ty(&body.local_decls, tcx);
                                        match callee_ty.kind() {
                                            ty::FnDef(callee, _) | ty::Closure(callee, _)=> {
                                                let id = call_graph.next_callsite_id();
                                                let callsite = CallSite::new(
                                                    //id,
                                                    did,
                                                    *callee,
                                                    //args.clone(),
                                                    //*destination,
                                                );

                                                call_graph.callsites.insert(id, Rc::new(callsite));
                                                call_graph
                                                    .function2callsite
                                                    .entry(*callee)
                                                    .and_modify(|set| {
                                                        set.insert(id);
                                                    })
                                                    .or_insert_with(|| {
                                                        let mut def = FxHashSet::default();
                                                        def.insert(id);
                                                        def
                                                    });
                                            },
                                            
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        call_graph
    }

    pub fn print_to_file(&self) {
        let mut readable_map = FxHashMap::default();
        for (id, callee) in &self.callsites {
            let callee_name = self.tcx.def_path(callee.caller()).to_string_no_crate_verbose();
            let caller_name = self.tcx.def_path(callee.callee()).to_string_no_crate_verbose();
            readable_map.insert(id, (caller_name, callee_name));
        }

        let path = Path::new("/tmp/metasafe/");
        if !path.exists() {
            let _ = std::fs::create_dir(path).unwrap();
        }

        let mut file = path.to_path_buf();
        file.push("callsites.json");

        let json_string = serde_json::to_string(&readable_map).unwrap();
        std::fs::write(file, json_string).unwrap();
    }
}
