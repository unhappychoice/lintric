// Result types, the question mark operator and matching on outcomes.

struct Config {
    retries: i32,
}

enum Failure {
    Missing,
}

fn load() -> Result<Config, Failure> { //~ depends: Config@3, Failure@7
    Ok(Config { retries: 1 }) //~ depends: Config@3, retries@4
}

fn retries() -> Result<i32, Failure> { //~ depends: Failure@7
    let config = load()?; //~ depends: load@11
    Ok(config.retries) //~ depends: config@16, retries@4
}

fn main() {
    match retries() { //~ depends: retries@15
        Ok(count) => count,
        Err(Failure::Missing) => 0, //~ depends: Failure@7, Missing@8
    };
}
