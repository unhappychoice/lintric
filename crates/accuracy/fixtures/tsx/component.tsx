// JSX element and component references.

const title = "lintric";

function Label(): JSX.Element {
    return <span>{title}</span>; //~ depends: title@3
}

function App(): JSX.Element {
    return <Label />; //~ depends: Label@5
}
