use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Meta};

/// Derive macro for builtin functions
///
/// Example:
/// ```ignore
/// #[derive(BuiltinFunction)]
/// #[builtin(name = "array.new_float")]
/// struct ArrayNewFloat {
///     size: f64,
///     initial_value: Value,
/// }
///
/// impl ArrayNewFloat {
///     fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
///         // implementation
///     }
/// }
/// ```
#[proc_macro_derive(BuiltinFunction, attributes(builtin, arg))]
pub fn builtin_function_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Parse the function name from attributes
    let function_name = parse_function_name(&input);

    let struct_name = &input.ident;

    // Extract field information
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("BuiltinFunction only works with named fields"),
        },
        _ => panic!("BuiltinFunction only works with structs"),
    };

    // Generate field parsing code
    let field_parsing = generate_field_parsing(fields);
    let field_validation = generate_field_validation(fields);
    let struct_construction = generate_struct_construction(fields);

    let expanded = quote! {
        impl #struct_name {
            pub fn builtin_fn(
                ctx: &mut ::pine_interpreter::Interpreter,
                args: Vec<::pine_interpreter::EvaluatedArg>,
            ) -> Result<::pine_interpreter::Value, ::pine_interpreter::RuntimeError> {
                use ::pine_interpreter::{Value, RuntimeError, EvaluatedArg};

                #field_parsing

                #field_validation

                let instance = Self {
                    #struct_construction
                };

                instance.execute(ctx)
            }

            pub fn name() -> &'static str {
                #function_name
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_function_name(input: &DeriveInput) -> String {
    for attr in &input.attrs {
        if let Meta::List(meta_list) = &attr.meta {
            if meta_list.path.is_ident("builtin") {
                let tokens_str = meta_list.tokens.to_string();
                if let Some(start) = tokens_str.find('"') {
                    if let Some(end) = tokens_str[start + 1..].find('"') {
                        return tokens_str[start + 1..start + 1 + end].to_string();
                    }
                }
            }
        }
    }
    panic!("BuiltinFunction requires a #[builtin(name = \"...\")] attribute");
}

fn generate_field_parsing(fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>) -> proc_macro2::TokenStream {
    let mut field_decls = Vec::new();
    let mut positional_matches = Vec::new();
    let mut named_matches = Vec::new();

    for (idx, field) in fields.iter().enumerate() {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type = &field.ty;

        // Parse attributes
        let (has_default, default_value) = parse_field_default(field);

        if has_default {
            if let Some(default_val) = default_value {
                field_decls.push(quote! {
                    let mut #field_name: #field_type = #default_val;
                });
            } else {
                field_decls.push(quote! {
                    let mut #field_name: #field_type = Default::default();
                });
            }
        } else {
            field_decls.push(quote! {
                let mut #field_name: Option<#field_type> = None;
            });
        }

        // Generate positional assignment based on type
        let positional_assign = generate_value_conversion(field_name, field_type, has_default);

        positional_matches.push(quote! {
            #idx => { #positional_assign }
        });

        // Generate named assignment
        let named_assign = generate_value_conversion(field_name, field_type, has_default);

        named_matches.push(quote! {
            #field_name_str => { #named_assign }
        });
    }

    quote! {
        #(#field_decls)*

        let mut positional_idx = 0;
        for arg in args {
            match arg {
                EvaluatedArg::Positional(arg_value) => {
                    match positional_idx {
                        #(#positional_matches)*
                        _ => return Err(RuntimeError::TypeError("Too many positional arguments".into()))
                    }
                    positional_idx += 1;
                }
                EvaluatedArg::Named { name: param_name, value: arg_value } => {
                    match param_name.as_str() {
                        #(#named_matches)*
                        _ => return Err(RuntimeError::TypeError(format!("Unknown parameter: {}", param_name)))
                    }
                }
            }
        }
    }
}

fn parse_field_default(field: &Field) -> (bool, Option<proc_macro2::TokenStream>) {
    for attr in &field.attrs {
        if let Meta::List(meta_list) = &attr.meta {
            if meta_list.path.is_ident("arg") {
                let tokens_str = meta_list.tokens.to_string();
                if tokens_str.contains("default") {
                    // Try to extract default value
                    if let Some(eq_pos) = tokens_str.find('=') {
                        let default_str = tokens_str[eq_pos + 1..].trim();
                        if !default_str.is_empty() && default_str != "\"\"" {
                            let default_tokens: proc_macro2::TokenStream = default_str.parse().unwrap();
                            // Check if field type is String and default is a string literal
                            let type_str = quote! { #field.ty }.to_string();
                            if type_str.contains("String") && default_str.starts_with('"') {
                                return (true, Some(quote! { #default_tokens.to_string() }));
                            }
                            return (true, Some(quote! { #default_tokens }));
                        }
                    }
                    return (true, None);
                }
            }
        }
    }
    (false, None)
}

fn generate_value_conversion(
    field_name: &syn::Ident,
    field_type: &syn::Type,
    has_default: bool,
) -> proc_macro2::TokenStream {
    let type_str = quote! { #field_type }.to_string();

    let conversion = if type_str.contains("f64") {
        quote! { arg_value.as_number()? }
    } else if type_str.contains("String") {
        quote! { arg_value.as_string()? }
    } else if type_str.contains("bool") {
        quote! { arg_value.as_bool()? }
    } else if type_str.contains("Value") {
        quote! { arg_value }
    } else {
        // Fallback
        quote! { arg_value }
    };

    if has_default {
        quote! {
            #field_name = #conversion;
        }
    } else {
        quote! {
            #field_name = Some(#conversion);
        }
    }
}

fn generate_field_validation(fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>) -> proc_macro2::TokenStream {
    let mut validations = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        // Check if field has a default
        let has_default = field.attrs.iter().any(|attr| {
            if let Meta::List(meta_list) = &attr.meta {
                meta_list.path.is_ident("arg") &&
                meta_list.tokens.to_string().contains("default")
            } else {
                false
            }
        });

        if !has_default {
            validations.push(quote! {
                let #field_name = #field_name.ok_or_else(|| {
                    RuntimeError::TypeError(format!("Missing required parameter: {}", #field_name_str))
                })?;
            });
        }
    }

    quote! {
        #(#validations)*
    }
}

fn generate_struct_construction(fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>) -> proc_macro2::TokenStream {
    let field_assignments: Vec<_> = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote! { #field_name }
    }).collect();

    quote! {
        #(#field_assignments),*
    }
}
