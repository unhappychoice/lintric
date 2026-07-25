// Static members, accessors and private fields.

class Counter {
    static LIMIT = 10;

    #count = 0;

    get current(): number {
        return this.#count; //~ depends: count@6
    }

    set current(next: number) {
        this.#count = next; //~ depends: count@6, next@12
    }

    atLimit(): boolean {
        return this.#count >= Counter.LIMIT; //~ depends: count@6, Counter@3, LIMIT@4
    }
}

const counter = new Counter(); //~ depends: Counter@3
const done = counter.atLimit(); //~ depends: atLimit@16, counter@21
