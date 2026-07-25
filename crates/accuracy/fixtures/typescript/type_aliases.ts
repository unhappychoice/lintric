// Type aliases, unions and intersections.

interface Named {
    name: string;
}

interface Aged {
    age: number;
}

type Person = Named & Aged; //~ depends: Named@3, Aged@7

type Maybe = Person | null; //~ depends: Person@11

function describe(person: Person): string { //~ depends: Person@11
    return person.name; //~ depends: person@15, name@4
}

function pick(value: Maybe): string { //~ depends: Maybe@13
    return value === null ? "" : describe(value); //~ depends: value@19, describe@15
}
