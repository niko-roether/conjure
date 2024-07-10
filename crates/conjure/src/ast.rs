pub enum SimpleType {
    Truth,
    NaturalCount,
    WholeCount,
    Amount,
    Phrase,
}

pub struct ConjoinedType {
    pub tys: Vec<Type>,
}

pub struct CharmType {
    pub components: ConjoinedType,
    pub product: Box<Type>,
}

pub struct SpellType {
    pub components: ConjoinedType,
    pub product: Box<Type>,
}

pub enum Type {
    Inferred,
    Symbol(Symbol),
    Simple(SimpleType),
    Conjoined(ConjoinedType),
    Charm(CharmType),
    Spell(SpellType),
}

pub struct Manifest {
    pub name: String,
    pub ty: Option<SimpleType>,
}

pub struct Symbol {
    pub name: String,
}

pub enum Element {
    Truth(bool),
    NaturalCount(u64),
    WholeCount(i64),
    Amount(f64),
    Phrase(String),
}

pub enum Value {
    Element(Element),
    Symbol(Symbol),
    Boundary(Boundary),
    Charm(Charm),
    Spell(Spell),
    Type(Type),
}

pub struct Binding {
    pub manifest: Manifest,
    pub value: Value,
}

pub struct Boundary {
    pub bindings: Vec<Binding>,
    pub value: Box<Value>,
}

pub struct Charm {
    pub components: Vec<Manifest>,
    pub boundary: Boundary,
}

pub struct Cast {
    pub symbol: Symbol,
    pub components: Vec<Value>,
}

pub struct ActionSequence {
    pub spells: Vec<Action>,
}

pub struct Spell {
    pub components: Vec<Manifest>,
    pub actions: ActionSequence,
}

pub enum Action {
    Cast(Cast),
    Binding(Binding),
    Sequence(ActionSequence),
}
