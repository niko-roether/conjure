use std::f64;

use bounding::ShapeMut;
// use conjure::ast::{Action, Binding, Cast, Conjuration, Element, Manifest, Spell, Type, Value};
use nalgebra::vector;
// use visual::Figure;

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
    // let mut weird_rect = bounding::Rect::from_width_height(2.0, 1.0);
    // weird_rect.set_center(vector![-0.5, 0.0]);
    // weird_rect.rotate(0.25 * f64::consts::TAU);
    // dbg!(&weird_rect);

    // let polygon = bounding::RegularPolygon::<5>::wrap(&weird_rect, 0.8, 0.0);
    // dbg!(&polygon);

    // let filling_circle = bounding::Circle::fill(&weird_rect, 0.0);
    // dbg!(&filling_circle);

    let rect = bounding::Rect::from_width_height(1.0, 1.0);
    let triangle = bounding::RegularPolygon::<3>::fill(&rect, 0.3, 0.0);
    dbg!(&rect);
    dbg!(&triangle);
}
