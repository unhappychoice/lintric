// Generic interfaces, generic inheritance and their implementations.

interface Base<T> {
    run(value: T): number; //~ depends: T@3
}

interface Extended<T> extends Base<T> { //~ depends: Base@3
    extra(): number;
}

class Impl implements Extended<number> { //~ depends: Extended@7
    run(value: number): number { //~ depends: run@4
        return value; //~ depends: value@12
    }

    extra(): number { //~ depends: extra@8
        return 1;
    }
}

const impl = new Impl(); //~ depends: Impl@11
const total = impl.extra(); //~ depends: extra@16, impl@21
