// One name declared in both namespaces.
//
// TypeScript keeps types and values apart, so an interface and a `const` may share a name and each is
// invisible where the other belongs. A class, an enum and a namespace declare in both, which is why
// the rule is about what a declaration introduces rather than about matching like to like. See #259.

interface Setting {
    level: number;
}

const Setting = { level: 1 };

function read(s: Setting): number { //~ depends: Setting@7
    return s.level + Setting.level; //~ depends: s@13, level@8, Setting@11
}

class Both {
    kind = "both";
}

type Alias = Both; //~ depends: Both@17

function useClass(b: Both): string { //~ depends: Both@17
    const made: Both = new Both(); //~ depends: Both@17
    return b.kind + made.kind; //~ depends: b@23, kind@18, made@24
}
