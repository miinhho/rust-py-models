use super::types::{TypeExpr, TypeSpec};
use crate::{Dependency, ExportError};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

/// Options passed to Python's [`dataclasses.dataclass`](https://docs.python.org/3/library/dataclasses.html#dataclasses.dataclass).
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct DataclassOptions {
    /// Generates an immutable dataclass when `Some(true)`.
    pub frozen: Option<bool>,
    /// Generates a slotted dataclass when `Some(true)`.
    pub slots: Option<bool>,
    /// Makes generated constructor parameters keyword-only when `Some(true)`.
    pub kw_only: Option<bool>,
}

/// A Python default attached to a generated field.
#[derive(Clone, Eq, PartialEq)]
pub enum FieldDefault {
    /// Emits `dataclasses.field` with the supplied constructor behavior and value expression.
    DataclassField {
        /// Whether the field participates in the generated constructor.
        init: bool,
        /// Python source used as the field's default value.
        value: String,
    },
}

/// A field in a generated Python dataclass.
#[must_use = "field specifications must be attached to a declaration"]
#[derive(Clone, Eq, PartialEq)]
pub struct FieldSpec {
    /// Python attribute name.
    pub name: String,
    /// Python annotation and its import requirements.
    pub ty: TypeSpec,
    /// Optional Python default expression.
    pub default: Option<FieldDefault>,
    /// Documentation emitted for the Python attribute.
    pub documentation: Option<String>,
}

impl FieldSpec {
    /// Creates a required field with no generated documentation.
    pub fn new(name: impl Into<String>, ty: TypeSpec) -> Self {
        Self {
            name: name.into(),
            ty,
            default: None,
            documentation: None,
        }
    }

    /// Sets the Python default used for this field.
    pub fn with_default(mut self, default: FieldDefault) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets documentation emitted after the Python attribute declaration.
    pub fn with_documentation(mut self, documentation: Option<String>) -> Self {
        self.documentation = documentation;
        self
    }
}

/// A complete Python dataclass declaration.
#[derive(Clone, Eq, PartialEq)]
pub struct DataclassSpec {
    /// Python class name.
    pub name: String,
    /// Python class documentation.
    pub documentation: Option<String>,
    /// Dataclass decorator options.
    pub options: DataclassOptions,
    /// Generic parameters declared by the class.
    pub type_params: Vec<String>,
    /// Attributes in declaration order.
    pub fields: Vec<FieldSpec>,
}

/// A member of a generated string enum.
#[derive(Clone, Eq, PartialEq)]
pub struct EnumMemberSpec {
    /// Python enum member name.
    pub name: String,
    /// String value assigned to the member.
    pub value: String,
    /// Documentation emitted for the Python attribute.
    pub documentation: Option<String>,
}

/// A generated Python `str` enum declaration.
#[derive(Clone, Eq, PartialEq)]
pub struct StringEnumSpec {
    /// Python enum class name.
    pub name: String,
    /// Python class documentation.
    pub documentation: Option<String>,
    /// Members in declaration order.
    pub members: Vec<EnumMemberSpec>,
}

/// A top-level declaration emitted into a Python module.
#[derive(Clone, Eq, PartialEq)]
pub enum Declaration {
    /// A `typing.TypeVar` declaration.
    TypeVar(String),
    /// A Python dataclass.
    Dataclass(DataclassSpec),
    /// A Python `str` enum.
    StringEnum(StringEnumSpec),
    /// A `typing.TypeAlias` assignment.
    TypeAlias {
        /// Alias name.
        name: String,
        /// Type expression assigned to the alias.
        target: TypeSpec,
        /// Documentation emitted immediately before the assignment.
        documentation: Option<String>,
    },
    /// A `typing.NewType` assignment.
    NewType {
        /// New type name.
        name: String,
        /// Runtime type accepted by the new type callable.
        target: TypeSpec,
        /// Documentation emitted immediately before the assignment.
        documentation: Option<String>,
    },
}

impl Declaration {
    /// Returns the Python name introduced by this declaration.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::TypeVar(name)
            | Self::Dataclass(DataclassSpec { name, .. })
            | Self::StringEnum(StringEnumSpec { name, .. })
            | Self::TypeAlias { name, .. }
            | Self::NewType { name, .. } => name,
        }
    }
    pub(crate) fn type_specs(&self) -> impl Iterator<Item = &TypeSpec> {
        let fields = match self {
            Self::Dataclass(dataclass) => dataclass
                .fields
                .iter()
                .map(|field| &field.ty)
                .collect::<Vec<_>>(),
            Self::TypeAlias { target, .. } | Self::NewType { target, .. } => vec![target],
            Self::TypeVar(_) | Self::StringEnum(_) => Vec::new(),
        };
        fields.into_iter()
    }
}

/// Describes one Rust model and the Python declarations it exports.
///
/// `ModelSpec` is the low-level contract used by [`crate::PY::model_spec`].
/// Most users obtain it through `#[derive(PY)]`.
#[must_use = "model specifications must be returned or exported"]
pub struct ModelSpec {
    id: &'static str,
    concrete_id: &'static str,
    name: String,
    output_path: PathBuf,
    declarations: Vec<Declaration>,
    hashable: bool,
}

impl ModelSpec {
    /// Creates an empty model specification for Rust type `T`.
    ///
    /// `id` identifies the declaration across the dependency graph, while `name`
    /// and `output_path` select its Python name and module.
    pub fn new<T: ?Sized>(
        id: &'static str,
        name: impl Into<String>,
        output_path: impl Into<PathBuf>,
        hashable: bool,
    ) -> Self {
        Self {
            id,
            concrete_id: std::any::type_name::<T>(),
            name: name.into(),
            output_path: output_path.into(),
            declarations: Vec::new(),
            hashable,
        }
    }

    /// Appends a top-level Python declaration to this model.
    pub fn push(&mut self, declaration: Declaration) {
        self.declarations.push(declaration);
    }

    /// Appends a declaration and returns the updated model.
    pub fn with_declaration(mut self, declaration: Declaration) -> Self {
        self.push(declaration);
        self
    }

    /// Returns the primary dataclass fields used by Serde flattening.
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::NotFlattenable`] if the model's primary
    /// declaration is not a dataclass.
    pub fn flattened_fields(&self) -> Result<Vec<FieldSpec>, ExportError> {
        self.declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Dataclass(dataclass) if dataclass.name == self.name => {
                    Some(dataclass.fields.clone())
                }
                _ => None,
            })
            .ok_or(ExportError::NotFlattenable(self.concrete_id))
    }

    /// Returns whether instances of the generated Python type are hashable.
    #[must_use]
    pub fn is_hashable(&self) -> bool {
        self.hashable
    }

    /// Returns the stable identity used to deduplicate this declaration.
    #[must_use]
    pub fn id(&self) -> &'static str {
        self.id
    }

    /// Returns the concrete Rust type identity for this instantiation.
    #[must_use]
    pub fn concrete_id(&self) -> &'static str {
        self.concrete_id
    }

    /// Returns the primary Python declaration name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the module path relative to the export directory.
    #[must_use]
    pub fn output_path(&self) -> &Path {
        &self.output_path
    }

    pub(crate) fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    pub(crate) fn dependencies(&self) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        self.for_each_type_spec(|spec| dependencies.extend(spec.dependencies().iter().cloned()));
        dependencies
    }

    pub(crate) fn imports(&self) -> BTreeSet<String> {
        let mut imports = BTreeSet::new();
        self.for_each_type_spec(|spec| imports.extend(spec.imports().iter().cloned()));
        imports
    }

    pub(crate) fn definitions(&self) -> Result<Vec<Declaration>, ExportError> {
        let mut definitions = BTreeMap::<String, Declaration>::new();
        let mut conflict = None;
        self.for_each_type_spec(|spec| {
            for definition in spec.definitions() {
                let name = definition.name().to_owned();
                match definitions.get(&name) {
                    Some(existing) if existing != definition => {
                        conflict.get_or_insert(name);
                    }
                    Some(_) => {}
                    None => {
                        definitions.insert(name, definition.clone());
                    }
                }
            }
        });
        let local_names = self
            .declarations
            .iter()
            .map(Declaration::name)
            .collect::<HashSet<_>>();
        for name in definitions.keys() {
            if local_names.contains(name.as_str()) {
                conflict = Some(name.clone());
                break;
            }
        }
        match conflict {
            Some(name) => Err(ExportError::NameCollision(name)),
            None => Ok(definitions.into_values().collect()),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), ExportError> {
        let mut declaration_names = HashSet::new();
        for declaration in &self.declarations {
            let name = declaration.name();
            validate_identifier(name)?;
            if !declaration_names.insert(name) {
                return Err(ExportError::NameCollision(name.to_owned()));
            }
            match declaration {
                Declaration::TypeVar(_) => {}
                Declaration::Dataclass(dataclass) => {
                    let mut params = HashSet::new();
                    for param in &dataclass.type_params {
                        validate_identifier(param)?;
                        if !params.insert(param) {
                            return Err(ExportError::NameCollision(param.clone()));
                        }
                    }
                    let mut field_names = HashSet::new();
                    for field in &dataclass.fields {
                        validate_identifier(&field.name)?;
                        if !field_names.insert(&field.name) {
                            return Err(ExportError::NameCollision(field.name.clone()));
                        }
                        field.ty.validate()?;
                    }
                }
                Declaration::StringEnum(string_enum) => {
                    let mut member_names = HashSet::new();
                    let mut values = HashSet::new();
                    for member in &string_enum.members {
                        validate_identifier(&member.name)?;
                        if !member_names.insert(&member.name) || !values.insert(&member.value) {
                            return Err(ExportError::NameCollision(member.name.clone()));
                        }
                    }
                }
                Declaration::TypeAlias { target, .. } => {
                    if matches!(target.expression(), TypeExpr::Union(items) if items.is_empty()) {
                        return Err(ExportError::EmptyTypeAlias(name.to_owned()));
                    }
                    target.validate()?;
                }
                Declaration::NewType { target, .. } => target.validate()?,
            }
        }
        Ok(())
    }

    fn for_each_type_spec(&self, mut visit: impl FnMut(&TypeSpec)) {
        fn visit_declaration(declaration: &Declaration, visit: &mut impl FnMut(&TypeSpec)) {
            for spec in declaration.type_specs() {
                visit(spec);
                for definition in spec.definitions() {
                    visit_declaration(definition, visit);
                }
            }
        }

        for declaration in &self.declarations {
            visit_declaration(declaration, &mut visit);
        }
    }
}

fn validate_identifier(name: &str) -> Result<(), ExportError> {
    let mut chars = name.chars();
    let valid = matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    const KEYWORDS: &[&str] = &[
        "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
        "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
        "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
        "try", "while", "with", "yield",
    ];
    if valid && !KEYWORDS.contains(&name) {
        Ok(())
    } else {
        Err(ExportError::InvalidPythonIdentifier(name.to_owned()))
    }
}
