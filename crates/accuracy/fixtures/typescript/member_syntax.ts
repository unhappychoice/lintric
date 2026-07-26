// Ways of naming a member, and a decorator.
//
// Each of these was checked by hand while probing and was already correct; the fixture is what keeps
// it that way.

const registry = { first: 1 };
const key = "first";

function subscripts(): number {
    return registry[key] + registry["first"]; //~ depends: registry@6, key@7
}

function logged(target: unknown, name: string): void {}

class Decorated {
    #hidden = 1;

    @logged //~ depends: logged@13
    method(): void {}

    #compute(): number {
        return this.#hidden * 2; //~ depends: #hidden@16
    }

    total(): number {
        return this.#compute(); //~ depends: #compute@21
    }

    static origin = new Decorated(); //~ depends: Decorated@15
    static make(): Decorated { //~ depends: Decorated@15
        return Decorated.origin; //~ depends: Decorated@15, origin@29
    }
}

function isDecorated(v: unknown): v is Decorated { //~ depends: Decorated@15
    return v instanceof Decorated; //~ depends: v@35, Decorated@15
}
