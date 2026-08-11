// A getter and a setter of the same name are one member declared in two places.
//
// The receiver's type cannot tell them apart, since both belong to the same class. Direction can:
// reading reaches the getter, assigning reaches the setter. See #235.
//
// `+=` reads before it writes, so it reaches both; only the setter is recorded, because one usage
// records one target. That cost is pinned below rather than left implicit.

class Counter {
    private stored = 0;

    get value(): number {
        return this.stored; //~ depends: stored@10
    }

    set value(next: number) {
        this.stored = next; //~ depends: stored@10, next@16
    }

    read(): number {
        return this.value; //~ depends: value@12
    }

    write(): void {
        this.value = 5; //~ depends: value@16
    }

    bump(): void {
        this.value += 1; //~ depends: value@16
    }
}

const counter = new Counter(); //~ depends: Counter@9
const observed = counter.value; //~ depends: counter@33, value@12

// An interface declares a getter and a setter separately, and each is satisfied by its own half.
interface Measured {
    get value(): number;
    set value(next: number);
}

class Gauge implements Measured { //~ depends: Measured@37
    private held = 0;

    get value(): number { //~ depends: value@38
        return this.held; //~ depends: held@43
    }

    set value(next: number) { //~ depends: value@39
        this.held = next; //~ depends: held@43, next@49
    }
}
