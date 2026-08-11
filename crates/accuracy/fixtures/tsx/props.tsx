// Component props and nested JSX elements.

interface LabelProps {
    text: string;
}

function Label(props: LabelProps): JSX.Element { //~ depends: LabelProps@3
    return <span>{props.text}</span>; //~ depends: props@7, text@4
}

const greeting = "hello";

function App(): JSX.Element {
    // The attribute name references the declared prop: renaming it at line 4 breaks this line.
    return <Label text={greeting} />; //~ depends: Label@7, greeting@11, text@4
}
