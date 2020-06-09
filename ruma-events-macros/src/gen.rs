//! Details of generating code for the `ruma_event` procedural macro.

#![allow(dead_code)]

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    Attribute, Field, Ident, LitStr, Token,
};

use crate::parse::{Content, EventKind, RumaEventInput};

/// The result of processing the `ruma_event` macro, ready for output back to source code.
pub struct RumaEvent {
    /// Outer attributes on the field, such as a docstring.
    attrs: Vec<Attribute>,

    /// Information for generating the type used for the event's `content` field.
    content: Content,

    /// The name of the type of the event's `content` field.
    content_name: Ident,

    /// The variant of `ruma_events::EventType` for this event, determined by the `event_type`
    /// field.
    event_type: LitStr,

    /// Struct fields of the event.
    fields: Vec<Field>,

    /// The kind of event.
    kind: EventKind,

    /// The name of the event.
    name: Ident,
}

impl From<RumaEventInput> for RumaEvent {
    fn from(input: RumaEventInput) -> Self {
        let kind = input.kind;
        let name = input.name;
        let content_name = format_ident!("{}Content", name, span = Span::call_site());
        let event_type = input.event_type;

        let mut fields =
            populate_event_fields(content_name.clone(), input.fields.unwrap_or_else(Vec::new));

        fields.sort_unstable_by_key(|field| field.ident.clone().unwrap());

        Self {
            attrs: input.attrs,
            content: input.content,
            content_name,
            event_type,
            fields,
            kind,
            name,
        }
    }
}

impl ToTokens for RumaEvent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // let attrs = &self.attrs;
        let content_name = &self.content_name;
        // let event_fields = &self.fields;
        // let event_type = &self.event_type;

        let name = &self.name;
        let content_docstring = format!("The payload for `{}`.", name);

        let content = match &self.content {
            Content::Struct(fields) => {
                quote! {
                    #[doc = #content_docstring]
                    #[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
                    pub struct #content_name {
                        #(#fields),*
                    }
                }
            }
            Content::Typedef(typedef) => {
                let content_attrs = &typedef.attrs;
                let path = &typedef.path;

                quote! {
                    #(#content_attrs)*
                    pub type #content_name = #path;
                }
            }
        };

        content.to_tokens(tokens);
    }
}

/// Fills in the event's struct definition with fields common to all basic events.
fn populate_event_fields(content_name: Ident, mut fields: Vec<Field>) -> Vec<Field> {
    let punctuated_fields: Punctuated<ParsableNamedField, Token![,]> = parse_quote! {
        /// The event's content.
        pub content: #content_name,
    };

    fields.extend(punctuated_fields.into_iter().map(|p| p.field));

    fields
}

/// A wrapper around `syn::Field` that makes it possible to parse `Punctuated<Field, Token![,]>`
/// from a `TokenStream`.
///
/// See https://github.com/dtolnay/syn/issues/651 for more context.
struct ParsableNamedField {
    /// The wrapped `Field`.
    pub field: Field,
}

impl Parse for ParsableNamedField {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let field = Field::parse_named(input)?;

        Ok(Self { field })
    }
}
