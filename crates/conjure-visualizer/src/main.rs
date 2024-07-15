// use conjure::ast::{Action, Binding, Cast, Conjuration, Element, Manifest, Spell, Type, Value};
use nalgebra::vector;
// use visual::Figure;

use bounding::ShapeOps;

mod bounding;
mod layout;
mod visual;

fn main() {
    // let ast = Conjuration {
    //     bindings: vec![Binding {
    //         manifest: Manifest {
    //             symbol: "*".to_string(),
    //             ty: Type::Inferred,
    //         },
    //         value: Value::Spell(Spell {
    //             components: vec![],
    //             actions: vec![Action::Cast(Cast {
    //                 spell: Box::new(Action::Value(Value::Symbol("utter".to_string()))),
    //                 components: vec![Action::Value(Value::Element(Element::Phrase(
    //                     "Hello World!".to_string(),
    //                 )))],
    //             })],
    //             ty: Type::Nil,
    //         }),
    //     }],
    // };

    // dbg!(&ast);
    // dbg!(Figure::from(ast));
    //
    let mut weird_rect =
        bounding::Polygon::new_rect(vector![-1.0, -1.0], vector![2.0, 1.0]).unwrap();
    weird_rect.rotate(-1.2);
    let polygon = weird_rect.containing_regular_polygon(5, 0.4, 0.0);
    dbg!(weird_rect);
    dbg!(polygon);
}
