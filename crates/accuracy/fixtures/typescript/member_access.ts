// Member access told apart by the receiver's type.
//
// Two declarations sharing a member name are distinguished by what the receiver is: a binding
// annotated where it is declared, or `this` inside a class. Where the file does not state the
// receiver's type the access is left unresolved rather than pointed at both declarations.

interface Reader {
    label: string;
}

class Writer {
    label: number;

    describe(): string {
        return String(this.label); //~ depends: label@12
    }
}

function fromParameter(reader: Reader): string { //~ depends: Reader@7
    return reader.label; //~ depends: reader@19, label@8
}

function fromVariable(): number {
    const writer: Writer = new Writer(); //~ depends: Writer@11
    return writer.label; //~ depends: writer@24, label@12
}

function fromUnannotated(untyped: Reader | Writer): string { //~ depends: Reader@7, Writer@11
    return String(untyped.label); //~ depends: untyped@28, label@8, label@12
}

function throughWrappers(maybe?: Reader): void { //~ depends: Reader@7
    void maybe?.label; //~ depends: maybe@32, label@8
    void maybe!.label; //~ depends: maybe@32, label@8
    void (maybe).label; //~ depends: maybe@32, label@8
}

function statedAtTheAccess(unknownType: Reader | Writer): void { //~ depends: Reader@7, Writer@11
    void (unknownType as Writer).label; //~ depends: unknownType@38, Writer@11, label@12
}
