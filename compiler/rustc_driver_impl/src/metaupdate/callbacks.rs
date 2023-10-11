use crate::{
    Callbacks,
    Compilation
};

use super::MetaUpdateCallbacks;

impl Callbacks for MetaUpdateCallbacks {
    /// Visit the AST, this should be done after the HIR analysis
    /// So we don't do anything here
    fn after_parsing<'tcx>(
            &mut self,
            _compiler: &rustc_interface::interface::Compiler,
            _queries: &'tcx rustc_interface::Queries<'tcx>,
        ) -> Compilation {
        Compilation::Continue
    }

    /// This is after HIR analysis. Here we should perform our type analysis, 
    /// then reparse to re-obtain AST, modify the AST and perform HIR analysis again.
    fn after_analysis<'tcx>(
            &mut self,
            _handler: &rustc_session::EarlyErrorHandler,
            _compiler: &rustc_interface::interface::Compiler,
            _queries: &'tcx rustc_interface::Queries<'tcx>,
        ) -> Compilation {
        Compilation::Continue
    }

    /// We don't need to do anything here. Just like in after_parsing.
    fn after_expansion<'tcx>(
            &mut self,
            _compiler: &rustc_interface::interface::Compiler,
            _queries: &'tcx rustc_interface::Queries<'tcx>,
        ) -> Compilation {
        Compilation::Continue
    }
}
