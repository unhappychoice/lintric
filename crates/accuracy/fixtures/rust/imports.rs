// Module declarations, imports and qualified paths.
//
// A `use` statement creates a local binding, so a reference resolves to the import line and the
// import line resolves to the definition. See "Import resolution" in the crate README.

mod geometry {
    pub struct Rect {
        pub width: i32,
    }

    pub fn unit() -> Rect { //~ depends: Rect@7
        Rect { width: 1 } //~ depends: Rect@7, width@8
    }
}

use geometry::Rect; //~ depends: geometry@6, Rect@7

fn main() {
    let direct = geometry::unit(); //~ depends: geometry@6, unit@11
    let imported = Rect { width: 2 }; //~ depends: Rect@16, width@8
    let _ = direct.width + imported.width; //~ depends: direct@19, imported@20, width@8
}
