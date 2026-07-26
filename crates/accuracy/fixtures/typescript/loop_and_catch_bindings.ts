// Names introduced by a loop header or a catch clause.
//
// A `for` with `const`, `let` or `var` declares its variable; one without assigns to a binding that
// already exists, so it reads instead. A catch parameter always declares.
//
// Each name below collides deliberately with an outer declaration, so a name read as a reference
// when it declares would show up as an edge to the wrong line. See #241.

const item = 100;
let existing = 0;
const failure = "outer";

function declaring(values: number[]): number {
    let total = 0;

    for (const item of values) { //~ depends: values@13
        total += item; //~ depends: total@14, item@16
    }

    for (const key in values) { //~ depends: values@13
        total += Number(key); //~ depends: total@14, key@20
    }

    return total; //~ depends: total@14
}

function assigning(values: number[]): number {
    for (existing of values) { //~ depends: existing@10, values@27
        break;
    }

    return existing; //~ depends: existing@10
}

function catching(): string {
    try {
        return failure; //~ depends: failure@11
    } catch (failure) {
        return String(failure); //~ depends: failure@38
    }
}
