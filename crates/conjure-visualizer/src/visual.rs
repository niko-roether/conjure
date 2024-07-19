use conjure::ast;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CirclePattern {
    None,
    ConcentricLines,
    StrokeTriangles,
    FillTriangles,
    Dots,
    Runes,
    Rings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokePattern {
    Line,
    Chain,
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub stroke: StrokePattern,
    pub pattern: CirclePattern,
    pub double: bool,
    pub rim: Vec<Figure>,
    pub content: Box<Figure>,
}

#[derive(Debug, Clone)]
pub struct RegularPolygon {
    pub sides: usize,
    pub stroke: StrokePattern,
    pub content: Box<Figure>,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub items: Vec<Figure>,
    pub stroke: StrokePattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorationKind {
    Tilde,
    Hat,
    Rays,
}

#[derive(Debug, Clone)]
pub struct Decorated {
    pub kind: DecorationKind,
    pub content: Box<Figure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialShapeKind {
    Pentagram,
}

#[derive(Debug, Clone)]
pub struct Pentagram {
    pub content: Box<Figure>,
}

#[derive(Debug, Clone)]
pub struct Symbol(pub String);

#[derive(Debug, Clone)]
pub struct Phrase(pub String);

#[derive(Debug, Clone)]
pub enum Figure {
    Symbol(Symbol),
    Phrase(Phrase),
    Pentagram(Pentagram),
    Circle(Circle),
    RegularPolygon(RegularPolygon),
    Decorated(Decorated),
    Link(Link),
    Arrangement(Vec<Figure>),
}

impl From<ast::Type> for CirclePattern {
    fn from(value: ast::Type) -> Self {
        match value {
            ast::Type::Inferred => CirclePattern::None,
            ast::Type::Nil => CirclePattern::Rings,
            _ => todo!("Type {:?} not yet supported!", value),
        }
    }
}

impl From<ast::Manifest> for Figure {
    fn from(value: ast::Manifest) -> Self {
        Figure::Decorated(Decorated {
            kind: DecorationKind::Hat,
            content: Box::new(Figure::Circle(Circle {
                stroke: StrokePattern::Line,
                double: false,
                pattern: value.ty.into(),
                rim: vec![],
                content: Box::new(Figure::Symbol(Symbol(value.symbol))),
            })),
        })
    }
}

impl From<ast::Element> for Figure {
    fn from(value: ast::Element) -> Self {
        match value {
            ast::Element::Phrase(phrase) => Figure::Phrase(Phrase(phrase)),
            _ => todo!("Element type not yet supported!"),
        }
    }
}

impl From<ast::Value> for Figure {
    fn from(value: ast::Value) -> Self {
        match value {
            ast::Value::Symbol(symbol) => Figure::Circle(Circle {
                stroke: StrokePattern::Line,
                double: false,
                pattern: CirclePattern::None,
                rim: vec![],
                content: Box::new(Figure::Symbol(Symbol(symbol))),
            }),
            ast::Value::Element(element) => Figure::RegularPolygon(RegularPolygon {
                sides: 5,
                stroke: StrokePattern::Line,
                content: Box::new(element.into()),
            }),
            ast::Value::Spell(spell) => spell.into(),
            _ => todo!("Value type not yet supported!"),
        }
    }
}

impl From<ast::Action> for Figure {
    fn from(value: ast::Action) -> Self {
        match value {
            ast::Action::Value(value) => value.into(),
            ast::Action::Cast(cast) => Figure::Circle(Circle {
                stroke: StrokePattern::Line,
                double: true,
                pattern: CirclePattern::None,
                rim: cast.components.into_iter().map(Into::into).collect(),
                content: Box::new(Figure::Pentagram(Pentagram {
                    content: Box::new((*cast.spell).into()),
                })),
            }),
            _ => todo!("Action type not yet supported!"),
        }
    }
}

impl From<ast::ActionSequence> for Figure {
    fn from(value: ast::ActionSequence) -> Self {
        if value.len() == 1 {
            return value.into_iter().next().unwrap().into();
        }
        todo!("Only action sequences of length 1 are supported for now")
    }
}

impl From<ast::Spell> for Figure {
    fn from(value: ast::Spell) -> Self {
        Figure::Decorated(Decorated {
            kind: DecorationKind::Rays,
            content: Box::new(Figure::Circle(Circle {
                stroke: StrokePattern::Line,
                double: true,
                pattern: value.ty.into(),
                rim: value.components.into_iter().map(Into::into).collect(),
                content: Box::new(value.actions.into()),
            })),
        })
    }
}

impl From<ast::Binding> for Figure {
    fn from(value: ast::Binding) -> Self {
        Figure::Link(Link {
            items: vec![value.manifest.into(), value.value.into()],
            stroke: StrokePattern::Chain,
        })
    }
}

impl From<ast::Conjuration> for Figure {
    fn from(value: ast::Conjuration) -> Self {
        Figure::Arrangement(value.bindings.into_iter().map(Into::into).collect())
    }
}
