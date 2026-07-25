// Async functions, awaited calls and arrow functions.

interface Result {
    ok: boolean;
}

async function load(): Promise<Result> { //~ depends: Result@3
    return { ok: true };
}

async function check(): Promise<boolean> {
    const result = await load(); //~ depends: load@7
    return result.ok; //~ depends: result@12, ok@4
}

const wrap = async (): Promise<boolean> => {
    return check(); //~ depends: check@11
};

const pending = wrap(); //~ depends: wrap@16
