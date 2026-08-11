// Object literal shorthand, and members that share a name across interfaces.
//
// A shorthand property reads the binding it names but does not reference a declared member; see
// "Object shapes" in the crate README.
//
// The two interfaces deliberately share a member name, so `first.id` is told apart from
// `Second.id` only by the type the receiver is annotated with.

interface First {
    id: string;
}

interface Second {
    id: number;
}

const id = "x";
const shorthand = { id }; //~ depends: id@17
const explicit = { key: id }; //~ depends: id@17

function read(first: First): string { //~ depends: First@9
    return first.id; //~ depends: first@21, id@10
}

const value = read({ id: "y" }); //~ depends: read@21
const both = [shorthand, explicit, value]; //~ depends: shorthand@18, explicit@19, value@25
