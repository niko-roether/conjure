/// Any simple (primitive) type. These types are used as building blocks for more complex
/// types.
///
/// All simple types have associated elements (literals).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleType {
    Truth,
    NaturalCount,
    WholeCount,
    Amount,
    Phrase,
}

/// The type of a [`Charm`] (pure function). Consists of a [`ConjoinedType`] (tuple type) for the charm components
/// (parameters), and a product [`Type`] (return type).
///
#[derive(Debug, Clone)]
pub struct CharmType {
    pub components: ConjoinedType,
    pub product: Box<Type>,
}

/// The type of a [`Spell`] (imperative function). Consists of a [`ConjoinedType`] (tuple type) for the spell
/// components, and a product [`Type`] (return type).
#[derive(Debug, Clone)]
pub struct SpellType {
    pub components: ConjoinedType,
    pub product: Box<Type>,
}

/// The type of a [`Conjunction`] (tuple). Consists of a list of [`Type`]s.
pub type ConjoinedType = Vec<Type>;

/// The type of a [`Value`].
#[derive(Debug, Clone)]
pub enum Type {
    Inferred,
    Nil,
    Optional(Box<Type>),
    Symbol(Symbol),
    Simple(SimpleType),
    Conjoined(ConjoinedType),
    Charm(CharmType),
    Spell(SpellType),
    Type,
}

/// A declaration of a [`Symbol`] (identifier), that associates it with a [`Type`].
#[derive(Debug, Clone)]
pub struct Manifest {
    pub symbol: Symbol,
    pub ty: Type,
}

/// An identifier.
pub type Symbol = String;

/// A literal.
#[derive(Debug, Clone)]
pub enum Element {
    Nil,
    Truth(bool),
    NaturalCount(u64),
    WholeCount(i64),
    Amount(f64),
    Phrase(String),
}

/// A value.
#[derive(Debug, Clone)]
pub enum Value {
    Element(Element),
    Symbol(Symbol),
    Boundary(Boundary),
    Conjunction(Conjunction),
    Charm(Charm),
    Invocation(Invocation),
    Spell(Spell),
    Type(Type),
}

/// A tuple of values.
pub type Conjunction = Vec<Value>;

/// A constant definition.
#[derive(Debug, Clone)]
pub struct Binding {
    pub manifest: Manifest,
    pub value: Value,
}

/// A scope with associated [`Binding`]s (definitions) and a return [`Value`].
#[derive(Debug, Clone)]
pub struct Boundary {
    pub bindings: Vec<Binding>,
    pub value: Box<Value>,
    pub ty: Type,
}

/// A pure function. Consists of a list of [`Manifest`]s (declarations) for its components
/// (parameters), and its containing [`Boundary`] (scope).
#[derive(Debug, Clone)]
pub struct Charm {
    pub components: Vec<Manifest>,
    pub boundary: Boundary,
}

/// A call to a pure function. Consists of a [`Value`] that should resolve to a [`Charm`]
/// (pure function), and a list of [`Value`]s that return the charm components (parameters).
#[derive(Debug, Clone)]
pub struct Invocation {
    pub charm: Box<Value>,
    pub components: Vec<Value>,
}

/// A call to an imperative function. Consists of a [`Action`] that should return a [`Spell`]
/// (imperative function), and a list of [`Action`]s (imperative statements) that return the spell
/// components (parameters).
#[derive(Debug, Clone)]
pub struct Cast {
    pub spell: Box<Action>,
    pub components: Vec<Action>,
}

/// A sequence of [`Action`]s (imperative statements).
pub type ActionSequence = Vec<Action>;

/// An imperative function. Consists of a list of [`Manifest`]s (declarations) for its components
/// (parameters), and an [`ActionSequence`] (list of statements).
#[derive(Debug, Clone)]
pub struct Spell {
    pub components: Vec<Manifest>,
    pub actions: ActionSequence,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum Action {
    Value(Value),
    Cast(Cast),
    Binding(Binding),
}

#[derive(Debug, Clone)]
pub struct Conjuration {
    pub bindings: Vec<Binding>,
}
