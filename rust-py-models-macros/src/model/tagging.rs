use crate::attrs::Options;
use crate::python::python_ident;
use proc_macro2::Span;

#[derive(Clone)]
pub(super) enum Tagging {
    External,
    Internal(String),
    Adjacent { tag: String, content: String },
    Untagged,
}

impl Tagging {
    pub(super) fn from_options(options: &Options, span: Span) -> syn::Result<Self> {
        let tagging = match (
            options.serde_untagged,
            options.serde_tag.as_deref(),
            options.serde_content.as_deref(),
        ) {
            (false, None, None) => Self::External,
            (false, Some(tag), None) => Self::Internal(tag.to_owned()),
            (false, Some(tag), Some(content)) => Self::Adjacent {
                tag: tag.to_owned(),
                content: content.to_owned(),
            },
            (true, None, None) => Self::Untagged,
            (true, _, _) => {
                return Err(syn::Error::new(
                    span,
                    "serde untagged cannot be combined with tag or content",
                ));
            }
            (false, None, Some(_)) => {
                return Err(syn::Error::new(span, "serde content requires serde tag"));
            }
        };
        tagging.validate_field_names(span)?;
        Ok(tagging)
    }

    fn validate_field_names(&self, span: Span) -> syn::Result<()> {
        if let Self::Internal(tag) | Self::Adjacent { tag, .. } = self {
            python_ident(tag, span)?;
        }
        if let Self::Adjacent { content, .. } = self {
            python_ident(content, span)?;
        }
        Ok(())
    }
}
