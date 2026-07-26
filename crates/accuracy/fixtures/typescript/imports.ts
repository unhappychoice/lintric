// Import forms and the local names they introduce.
//
// The declarations these names refer to live in other files, which single-file analysis cannot see,
// so an import line has nothing to depend on. What it does is introduce a local name, and every
// reference in this file depends on the import line — the same half of the chain that
// `rust/imports.rs` records for a `use` statement.
//
// `helper as aid` introduces `aid` alone: `helper` is the other module's name for it and is not a
// local name at all. See #269.

import { Widget, helper as aid } from "./widgets";
import type { Shape } from "./shapes";
import Fallback from "./fallback";
import * as everything from "./all";

function build(s: Shape): Widget { //~ depends: Shape@12, Widget@11
    const made = new Widget(); //~ depends: Widget@11
    aid(made); //~ depends: aid@11, made@17
    Fallback.init(); //~ depends: Fallback@13
    everything.run(); //~ depends: everything@14
    return made; //~ depends: made@17
}

const built = build({ sides: 3 } as unknown as Shape); //~ depends: build@16, Shape@12
