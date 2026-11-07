#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

mod chams;
mod glapi;
mod printhook;
mod ratmesh;

use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};

static broken: AtomicBool = AtomicBool::new(false);

fn guard<T>(work: impl FnOnce() -> T) -> Option<T> {
    match panic::catch_unwind(AssertUnwindSafe(work)) {
        Ok(v) => Some(v),
        Err(_) => {
            broken.store(true, Ordering::Relaxed);
            None
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glCallList(list: u32) {
    let Some(o) = glapi::chain() else { return };

    if !broken.load(Ordering::Relaxed)
        && let Some(s) = glapi::state()
        && !unsafe { chams::iscompiling(s) }
    {
        let plan = guard(|| unsafe { ratmesh::plan(list) });
        if let Some(p) = plan
            && guard(|| unsafe { chams::drawlist(o, s, &p) }).is_some()
        {
            return;
        }
    }

    unsafe { (o.glCallList)(list) };
}
