use super::types::{TypeExpr, TypeSpec};
use crate::{Dependency, ExportError};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct DataclassOptions {
    pub frozen: Option<bool>,
    pub slots: Option<bool>,
    pub kw_only: Option<bool>,
}

#[derive(Clone, Eq, PartialEq)]
pub enum FieldDefault {
    DataclassField { init: bool, value: String },
}

#[must_use = "field specifications must be attached to a declaration"]
#[derive(Clone, Eq, PartialEq)]
pub struct FieldSpec {
    pub name: String,
    pub ty: TypeSpec,
    pub default: Option<FieldDefault>,
    pub documentation: Option<String>,
}

impl FieldSpec {
    pub fn new(name: impl Into<String>, ty: TypeSpec) -> Self {
        Self {
            name: name.into(),
            ty,
            default: None,
            documentation: None,
        }
    }

    pub fn with_default(mut self, default: FieldDefault) -> Self {
        self.default = Some(default);
        self
    }

    pub fn with_documentation(mut self, documentation: Option<String>) -> Self {
        self.documentation = documentation;
        self
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct DataclassSpec {
    pub name: String,
    pub documentation: Option<String>,
    pub options: DataclassOptions,
    pub type_params: Vec<String>,
    pub fields: Vec<FieldSpec>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct EnumMemberSpec {
    pub name: String,
    pub value: String,
    pub documentation: Option<String>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct StringEnumSpec {
    pub name: String,
    pub documentation: Option<String>,
    pub members: Vec<EnumMemberSpec>,
}

#[derive(Clone, Eq, PartialEq)]
pub enum Declaration {
    TypeVar(String),
    Dataclass(DataclassSpec),
    StringEnum(StringEnumSpec),
    TypeAlias {
        name: String,
        target: TypeSpec,
        documentation: Option<String>,
    },
    NewType {
        name: String,
        target: TypeSpec,
        documentation: Option<String>,
    },
}

impl Declaration {
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

    pub fn push(&mut self, declaration: Declaration) {
        self.declarations.push(declaration);
    }

    pub fn with_declaration(mut self, declaration: Declaration) -> Self {
        self.push(declaration);
        self
    }

    /// Return fields of the primary dataclass for Serde flattening.
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::NotFlattenable`] when this model is not a dataclass.
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

    #[must_use]
    pub fn is_hashable(&self) -> bool {
        self.hashable
    }

    #[must_use]
    pub fn id(&self) -> &'static str {
        self.id
    }

    #[must_use]
    pub fn concrete_id(&self) -> &'static str {
        self.concrete_id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

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
