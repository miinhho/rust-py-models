use super::model::Declaration;
use crate::{Dependency, ExportError};
use std::collections::BTreeSet;

/// A structured Python annotation expression.
///
/// Use [`TypeSpec`] constructors when defining a [`crate::PY`] implementation;
/// they retain the imports, dependencies, and validation required by the
/// expression.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TypeExpr {
    /// An unqualified Python name such as `str` or a model name.
    Name(String),
    /// A name accessed through an imported module, such as `datetime.date`.
    Qualified {
        /// Module imported by the generated file.
        module: String,
        /// Attribute read from the module.
        name: String,
    },
    /// A parameterized annotation such as `list[str]`.
    Subscript {
        /// Type constructor being parameterized.
        base: Box<Self>,
        /// Type arguments in source order.
        args: Vec<Self>,
    },
    /// A `|` union.
    Union(Vec<Self>),
    /// A fixed-length tuple annotation.
    Tuple(Vec<Self>),
    /// A homogeneous `tuple[T, ...]` annotation.
    VariadicTuple(Box<Self>),
    /// A `typing.Literal` value written as Python source.
    Literal(String),
    /// An unchecked Python annotation written verbatim.
    UnsafeRaw(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ValidationIssue {
    Unhashable(String),
    EmptyUnion,
}

/// A Python annotation together with everything needed to emit and validate it.
///
/// Besides the visible [`TypeExpr`], a specification carries module imports,
/// referenced Rust models, helper declarations, and Python hashability.
#[must_use = "type specifications must be consumed or attached to a model"]
#[derive(Clone, Eq, PartialEq)]
pub struct TypeSpec {
    expression: TypeExpr,
    validation_name: String,
    imports: BTreeSet<String>,
    definitions: Vec<Declaration>,
    dependencies: Vec<Dependency>,
    hashable: bool,
    issues: Vec<ValidationIssue>,
}

impl TypeSpec {
    /// Creates an annotation that refers to an unqualified Python name.
    pub fn named(name: impl Into<String>) -> Self {
        Self::new(TypeExpr::Name(name.into()))
    }

    /// Creates `module.name` and records the required module import.
    pub fn qualified(module: impl Into<String>, name: impl Into<String>) -> Self {
        let module = module.into();
        let mut spec = Self::new(TypeExpr::Qualified {
            module: module.clone(),
            name: name.into(),
        });
        spec.imports.insert(module);
        spec
    }

    /// Creates an unchecked annotation from verbatim Python source.
    ///
    /// This does not infer imports, discover model dependencies, or validate
    /// the expression. Prefer the structured constructors whenever possible.
    pub fn unsafe_raw(annotation: impl Into<String>) -> Self {
        Self::new(TypeExpr::UnsafeRaw(annotation.into()))
    }

    /// Creates a `typing.Literal` annotation from a Python value expression.
    pub fn literal(value: impl Into<String>) -> Self {
        Self::new(TypeExpr::Literal(value.into())).with_hashable(true)
    }

    /// Creates a symbolic name with the validation behavior of `actual`.
    ///
    /// Generic declarations use this to emit a `TypeVar` name while retaining
    /// the concrete type's hashability constraints.
    pub fn symbolic(name: impl Into<String>, actual: &Self) -> Self {
        let mut spec = Self::named(name);
        spec.validation_name.clone_from(&actual.validation_name);
        spec.hashable = actual.hashable;
        spec.issues.clone_from(&actual.issues);
        spec
    }

    /// Creates a Python `list[item]` annotation.
    pub fn list(item: Self) -> Self {
        Self::subscript(Self::named("list"), vec![item])
    }

    /// Creates a Python `collections.deque[item]` annotation.
    pub fn deque(item: Self) -> Self {
        Self::subscript(Self::qualified("collections", "deque"), vec![item])
    }

    /// Creates a Python `set[item]` annotation.
    ///
    /// Validation fails if `item` is known to be unhashable.
    pub fn set(item: Self) -> Self {
        let item_name = item.validation_name.clone();
        let item_hashable = item.hashable;
        let mut spec = Self::subscript(Self::named("set"), vec![item]);
        spec.hashable = false;
        if !item_hashable {
            spec.issues.push(ValidationIssue::Unhashable(item_name));
        }
        spec
    }

    /// Creates a Python `dict[key, value]` annotation.
    ///
    /// Validation fails if `key` is known to be unhashable.
    pub fn dict(key: Self, value: Self) -> Self {
        let key_name = key.validation_name.clone();
        let key_hashable = key.hashable;
        let mut spec = Self::subscript(Self::named("dict"), vec![key, value]);
        spec.hashable = false;
        if !key_hashable {
            spec.issues.push(ValidationIssue::Unhashable(key_name));
        }
        spec
    }

    /// Creates a fixed-length Python tuple annotation.
    pub fn tuple(items: Vec<Self>) -> Self {
        let hashable = items.iter().all(Self::is_hashable);
        let mut spec = Self::combine(
            TypeExpr::Tuple(items.iter().map(|item| item.expression.clone()).collect()),
            items,
        );
        spec.hashable = hashable;
        spec
    }

    /// Creates a homogeneous `tuple[item, ...]` annotation.
    pub fn variadic_tuple(item: Self) -> Self {
        let hashable = item.hashable;
        let expression = TypeExpr::VariadicTuple(Box::new(item.expression.clone()));
        let mut spec = Self::combine(expression, vec![item]);
        spec.hashable = hashable;
        spec
    }

    /// Creates a Python union.
    ///
    /// An empty input is retained as a validation error.
    pub fn union(items: Vec<Self>) -> Self {
        let empty = items.is_empty();
        let hashable = items.iter().all(Self::is_hashable);
        let mut spec = Self::combine(
            TypeExpr::Union(items.iter().map(|item| item.expression.clone()).collect()),
            items,
        );
        spec.hashable = hashable;
        if empty {
            spec.issues.push(ValidationIssue::EmptyUnion);
        }
        spec
    }

    /// Creates `item | None`.
    pub fn optional(item: Self) -> Self {
        Self::union(vec![item, Self::named("None").with_hashable(true)])
    }

    /// Applies type arguments to a Python type constructor.
    pub fn subscript(base: Self, args: Vec<Self>) -> Self {
        let mut children = Vec::with_capacity(args.len() + 1);
        children.push(base.clone());
        children.extend(args.iter().cloned());
        let expression = TypeExpr::Subscript {
            base: Box::new(base.expression),
            args: args.into_iter().map(|arg| arg.expression).collect(),
        };
        Self::combine(expression, children)
    }

    /// Refers to another generated model and records it as a dependency.
    pub fn model(
        name: impl Into<String>,
        args: Vec<Self>,
        dependency: Dependency,
        hashable: bool,
    ) -> Self {
        let base = Self::named(name);
        let mut spec = if args.is_empty() {
            base
        } else {
            Self::subscript(base, args)
        };
        spec.dependencies.push(dependency);
        spec.hashable = hashable;
        spec
    }

    /// Overrides whether the Python representation may be a set item or dict key.
    pub fn with_hashable(mut self, hashable: bool) -> Self {
        self.hashable = hashable;
        self
    }

    /// Adds an absolute Python module import required by this annotation.
    pub fn with_import(mut self, module: impl Into<String>) -> Self {
        self.imports.insert(module.into());
        self
    }

    /// Adds a helper declaration that must appear in the containing module.
    pub fn with_definition(mut self, definition: Declaration) -> Self {
        self.definitions.push(definition);
        self
    }

    /// Renders the annotation without surrounding declarations or imports.
    #[must_use]
    pub fn annotation(&self) -> String {
        super::render::render_type(&self.expression)
    }

    /// Returns whether the Python representation may be a set item or dict key.
    #[must_use]
    pub fn is_hashable(&self) -> bool {
        self.hashable
    }

    pub(crate) fn validate(&self) -> Result<(), ExportError> {
        match self.issues.first() {
            Some(ValidationIssue::Unhashable(name)) => {
                Err(ExportError::UnhashableType(name.clone()))
            }
            Some(ValidationIssue::EmptyUnion) => Err(ExportError::EmptyUnion),
            None => Ok(()),
        }
    }

    pub(crate) fn expression(&self) -> &TypeExpr {
        &self.expression
    }

    pub(crate) fn imports(&self) -> &BTreeSet<String> {
        &self.imports
    }

    pub(crate) fn definitions(&self) -> &[Declaration] {
        &self.definitions
    }

    pub(crate) fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    fn new(expression: TypeExpr) -> Self {
        let validation_name = super::render::render_type(&expression);
        Self {
            expression,
            validation_name,
            imports: BTreeSet::new(),
            definitions: Vec::new(),
            dependencies: Vec::new(),
            hashable: false,
            issues: Vec::new(),
        }
    }

    fn combine(expression: TypeExpr, children: Vec<Self>) -> Self {
        let mut spec = Self::new(expression);
        for child in children {
            spec.imports.extend(child.imports);
            spec.definitions.extend(child.definitions);
            spec.dependencies.extend(child.dependencies);
            spec.issues.extend(child.issues);
        }
        spec
    }
}
