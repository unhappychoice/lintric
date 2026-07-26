// Casts, `this` typing, instantiation, override and nested namespaces, plus what an export names.
//
// Each was checked by hand while probing and was already correct; the fixture is what keeps it that
// way.

interface Spec {
    size: number;
}

const raw = { size: 1 };

const checked = raw satisfies Spec; //~ depends: raw@10, Spec@6
const asserted = <Spec>raw; //~ depends: Spec@6, raw@10
const cast = raw as Spec; //~ depends: raw@10, Spec@6

class Chainable {
    value = 1;

    self(): this {
        return this;
    }
}

class Sub extends Chainable { //~ depends: Chainable@16
    override self(): this { //~ depends: self@19
        return this;
    }
}

function generic<T>(v: T): T {
    return v; //~ depends: v@30
}

const instantiated = generic<Spec>; //~ depends: generic@30, Spec@6

namespace Outer {
    export namespace Inner {
        export const deep = 1;
    }
}

const reached = Outer.Inner.deep; //~ depends: Outer@36, Inner@37

export { raw }; //~ depends: raw@10
export { raw as exposed }; //~ depends: raw@10
export type { Spec }; //~ depends: Spec@6
export default checked; //~ depends: checked@12
