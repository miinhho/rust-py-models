use super::model::Declaration;
use crate::{Dependency, ExportError};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TypeExpr {
    Name(String),
    Qualified { module: String, name: String },
    Subscript { base: Box<Self>, args: Vec<Self> },
    Union(Vec<Self>),
    Tuple(Vec<Self>),
    VariadicTuple(Box<Self>),
    Literal(String),
    UnsafeRaw(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ValidationIssue {
    Unhashable(String),
    EmptyUnion,
}

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
    pub fn named(name: impl Into<String>) -> Self {
        Self::new(TypeExpr::Name(name.into()))
    }

    pub fn qualified(module: impl Into<String>, name: impl Into<String>) -> Self {
        let module = module.into();
        let mut spec = Self::new(TypeExpr::Qualified {
            module: module.clone(),
            name: name.into(),
        });
        spec.imports.insert(module);
        spec
    }

    pub fn unsafe_raw(annotation: impl Into<String>) -> Self {
        Self::new(TypeExpr::UnsafeRaw(annotation.into()))
    }

    pub fn literal(value: impl Into<String>) -> Self {
        Self::new(TypeExpr::Literal(value.into())).with_hashable(true)
    }

    pub fn symbolic(name: impl Into<String>, actual: &Self) -> Self {
        let mut spec = Self::named(name);
        spec.validation_name.clone_from(&actual.validation_name);
        spec.hashable = actual.hashable;
        spec.issues.clone_from(&actual.issues);
        spec
    }

    pub fn list(item: Self) -> Self {
        Self::subscript(Self::named("list"), vec![item])
    }

    pub fn deque(item: Self) -> Self {
        Self::subscript(Self::qualified("collections", "deque"), vec![item])
    }

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

    pub fn tuple(items: Vec<Self>) -> Self {
        let hashable = items.iter().all(Self::is_hashable);
        let mut spec = Self::combine(
            TypeExpr::Tuple(items.iter().map(|item| item.expression.clone()).collect()),
            items,
        );
        spec.hashable = hashable;
        spec
    }

    pub fn variadic_tuple(item: Self) -> Self {
        let hashable = item.hashable;
        let expression = TypeExpr::VariadicTuple(Box::new(item.expression.clone()));
        let mut spec = Self::combine(expression, vec![item]);
        spec.hashable = hashable;
        spec
    }

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

    pub fn optional(item: Self) -> Self {
        Self::union(vec![item, Self::named("None").with_hashable(true)])
    }

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

    pub fn with_hashable(mut self, hashable: bool) -> Self {
        self.hashable = hashable;
        self
    }

    pub fn with_import(mut self, module: impl Into<String>) -> Self {
        self.imports.insert(module.into());
        self
    }

    pub fn with_definition(mut self, definition: Declaration) -> Self {
        self.definitions.push(definition);
        self
    }

    #[must_use]
    pub fn annotation(&self) -> String {
        super::render::render_type(&self.expression)
    }

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
