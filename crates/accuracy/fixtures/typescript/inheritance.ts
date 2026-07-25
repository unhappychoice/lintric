// Interface inheritance and class extension.

interface Base {
    run(): number;
}

interface Extended extends Base { //~ depends: Base@3
    extra(): number;
}

class First implements Extended { //~ depends: Extended@7
    run(): number { //~ depends: run@4
        return 1;
    }

    extra(): number { //~ depends: extra@8
        return 2;
    }
}

class Second extends First { //~ depends: First@11
    run(): number { //~ depends: run@12
        return 3;
    }
}
