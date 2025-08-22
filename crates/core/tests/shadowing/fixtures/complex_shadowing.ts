// Complex TypeScript shadowing test cases from Issue #103

function main() {
    let x = 1;  // outer x
    {
        let x = 2;  // shadows outer x
        let y = x;  // should reference inner x (value: 2)
    }
    let z = x;  // should reference outer x (value: 1)
}

function complexShadowing() {
    let variable = 10;           // level 1
    
    function innerFunc() {
        let variable = 20;       // level 2 - shadows level 1
        
        {
            let variable = 30;   // level 3 - shadows level 2
            console.log(variable);  // should use level 3 (value: 30)
            
            {
                console.log(variable);  // still level 3 (value: 30)
            }
        }
        
        console.log(variable);  // back to level 2 (value: 20)
    }
    
    console.log(variable);    // level 1 (value: 10)
    innerFunc();
}

function example(param: number) {
    let param = param + 1;  // shadows function parameter
    
    {
        let param = param * 2;  // shadows local variable
        console.log(param);   // should use innermost param
    }
    
    console.log(param);  // should use local variable (not original parameter)
}

function crossScopeTest() {
    let name = "outer";
    
    function inner1() {
        let name = "inner1";
        console.log(name);  // inner1
    }
    
    function inner2() {
        console.log(name);  // should use outer scope "outer"
    }
    
    {
        let name = "block";
        inner1();  // inner1's name doesn't affect this
        inner2();  // should still see outer "name"
        console.log(name);  // block
    }
    
    inner1();
    inner2();
    console.log(name);  // outer
}

// Four levels deep shadowing
function deepShadowing() {
    let deepVar = 1;  // level 1
    {
        let deepVar = 2;  // level 2
        {
            let deepVar = 3;  // level 3
            {
                let deepVar = 4;  // level 4
                {
                    let deepVar = 5;  // level 5
                    console.log(deepVar);  // should be 5
                }
                console.log(deepVar);  // should be 4
            }
            console.log(deepVar);  // should be 3
        }
        console.log(deepVar);  // should be 2
    }
    console.log(deepVar);  // should be 1
}

// Class and method shadowing
class TestClass {
    private classMember = "class";
    
    method() {
        let classMember = "method";  // shadows class member
        
        {
            let classMember = "block";  // shadows method variable
            console.log(classMember);  // should be "block"
        }
        
        console.log(classMember);  // should be "method"
        console.log(this.classMember);  // should be "class"
    }
}

// Arrow functions with shadowing
function arrowFunctionShadowing() {
    let arrowVar = "outer";
    
    const arrow1 = () => {
        let arrowVar = "arrow1";
        
        const arrow2 = () => {
            let arrowVar = "arrow2";
            console.log(arrowVar);  // should be "arrow2"
        };
        
        arrow2();
        console.log(arrowVar);  // should be "arrow1"
    };
    
    arrow1();
    console.log(arrowVar);  // should be "outer"
}

// Interface and type shadowing scenarios
interface OuterInterface {
    prop: string;
}

function interfaceShadowing() {
    type OuterInterface = { different: number };  // shadows interface
    
    let value: OuterInterface = { different: 42 };  // should use type, not interface
    
    {
        interface OuterInterface {  // shadows type
            another: boolean;
        }
        
        // This would create conflict in real TypeScript, but for testing shadowing resolution
        let innerValue: OuterInterface = { another: true };
    }
}

// Try-catch shadowing
function tryCatchShadowing() {
    let error = "outer";
    
    try {
        let error = "try";  // shadows outer
        throw new Error("test");
    } catch (error) {  // shadows try block's error
        let error = "catch";  // shadows catch parameter
        console.log(error);  // should be "catch"
    } finally {
        let error = "finally";  // shadows outer
        console.log(error);  // should be "finally"
    }
    
    console.log(error);  // should be "outer"
}

// Loop variable shadowing
function loopShadowing() {
    let i = 0;
    for (let i = 0; i < 5; i++) {  // shadows outer i
        let i = i * 2;  // shadows loop variable i (would be error in real TS)
        console.log(i);
    }
    console.log(i);  // original i
    
    let j = 10;
    for (let j in [1, 2, 3]) {  // shadows outer j
        let j = parseInt(j) * 2;  // shadows loop variable j
        console.log(j);
    }
    console.log(j);  // original j
}