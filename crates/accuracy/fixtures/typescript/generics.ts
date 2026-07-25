// Generic functions, generic classes and constraints.

interface Named {
    name: string;
}

class Box<T> {
    value: T; //~ depends: T@7

    constructor(value: T) { //~ depends: T@7
        this.value = value; //~ depends: value@8, value@10
    }
}

function label<T extends Named>(item: T): string { //~ depends: Named@3
    return item.name; //~ depends: item@15, name@4
}

const boxed = new Box<Named>("x"); //~ depends: Box@7, Named@3
const text = label({ name: "y" }); //~ depends: label@15
const held = boxed.value; //~ depends: boxed@19, value@8
