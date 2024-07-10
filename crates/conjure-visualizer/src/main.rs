use conjure::ast::{Action, Binding, Cast, Conjuration, Element, Manifest, Spell, Type, Value};
use visual::Figure;

mod layout;
mod visual;

fn main() {
    let ast = Conjuration {
        bindings: vec![Binding {
            manifest: Manifest {
                symbol: "*".to_string(),
                ty: Type::Inferred,
            },
            value: Value::Spell(Spell {
                components: vec![],
                actions: vec![Action::Cast(Cast {
                    spell: Box::new(Action::Value(Value::Symbol("utter".to_string()))),
                    components: vec![Action::Value(Value::Element(Element::Phrase(
                        "Hello World!".to_string(),
                    )))],
                })],
                ty: Type::Nil,
            }),
        }],
    };

    dbg!(&ast);
    dbg!(Figure::from(ast));
}
