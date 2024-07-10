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
pub enum CircleKind {
    Single,
    Double,
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub kind: CircleKind,
    pub pattern: CirclePattern,
    pub children: Vec<Figure>,
    pub content: Option<Box<Figure>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkPattern {
    Line,
    Chain,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub items: Vec<Figure>,
    pub pattern: LinkPattern,
}

#[derive(Debug, Clone)]
pub struct Decorated {
    pub tilde: bool,
    pub hat: bool,
    pub rays: bool,
    pub figure: Box<Figure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Star,
}

#[derive(Debug, Clone)]
pub struct Shape {
    pub kind: ShapeKind,
    pub content: Option<Box<Figure>>,
}

#[derive(Debug, Clone)]
pub enum Figure {
    Symbol(String),
    Phrase(String),
    Shape(Shape),
    Circle(Circle),
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
            tilde: false,
            hat: true,
            rays: false,
            figure: Box::new(Figure::Circle(Circle {
                kind: CircleKind::Single,
                pattern: value.ty.into(),
                children: vec![],
                content: Some(Box::new(Figure::Symbol(value.symbol))),
            })),
        })
    }
}

impl From<ast::Element> for Figure {
    fn from(value: ast::Element) -> Self {
        match value {
            ast::Element::Phrase(phrase) => Figure::Phrase(phrase),
            _ => todo!("Element type not yet supported!"),
        }
    }
}

impl From<ast::Value> for Figure {
    fn from(value: ast::Value) -> Self {
        match value {
            ast::Value::Symbol(symbol) => Figure::Circle(Circle {
                kind: CircleKind::Single,
                pattern: CirclePattern::None,
                children: vec![],
                content: Some(Box::new(Figure::Symbol(symbol))),
            }),
            ast::Value::Element(element) => Figure::Circle(Circle {
                kind: CircleKind::Single,
                pattern: CirclePattern::None,
                children: vec![],
                content: Some(Box::new(element.into())),
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
                kind: CircleKind::Double,
                pattern: CirclePattern::None,
                children: cast.components.into_iter().map(Into::into).collect(),
                content: Some(Box::new(Figure::Shape(Shape {
                    kind: ShapeKind::Star,
                    content: Some(Box::new((*cast.spell).into())),
                }))),
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
            hat: false,
            tilde: false,
            rays: true,
            figure: Box::new(Figure::Circle(Circle {
                kind: CircleKind::Double,
                pattern: value.ty.into(),
                children: value.components.into_iter().map(Into::into).collect(),
                content: Some(Box::new(value.actions.into())),
            })),
        })
    }
}

impl From<ast::Binding> for Figure {
    fn from(value: ast::Binding) -> Self {
        Figure::Link(Link {
            items: vec![value.manifest.into(), value.value.into()],
            pattern: LinkPattern::Chain,
        })
    }
}

impl From<ast::Conjuration> for Figure {
    fn from(value: ast::Conjuration) -> Self {
        Figure::Arrangement(value.bindings.into_iter().map(Into::into).collect())
    }
}
