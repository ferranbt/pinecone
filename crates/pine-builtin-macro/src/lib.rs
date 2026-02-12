use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, Meta};

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
/// // With type parameters:
/// #[derive(BuiltinFunction)]
/// #[builtin(name = "matrix.new", type_params = 1)]
/// struct MatrixNew {
///     #[type_param]
///     element_type: String,
///     rows: f64,
///     columns: f64,
///     initial_value: Value,
/// }
///
/// impl ArrayNewFloat {
///     fn execute(&self, ctx: &mut Interpreter) -> Result<Value, RuntimeError> {
///         // implementation
///     }
/// }
/// ```
#[proc_macro_derive(BuiltinFunction, attributes(builtin, arg, type_param))]
pub fn builtin_function_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Parse the function name and type params count from attributes
    let (function_name, type_params_count) = parse_builtin_attributes(&input);

    let struct_name = &input.ident;

    // Extract field information
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("BuiltinFunction only works with named fields"),
        },
        _ => panic!("BuiltinFunction only works with structs"),
    };

    // Generate type param extraction if needed
    let type_param_extraction = if type_params_count > 0 {
        generate_type_param_extraction(fields, type_params_count)
    } else {
        quote! {}
    };

    // Generate field parsing code
    let field_parsing = generate_field_parsing(fields);
    let field_validation = generate_field_validation(fields);
    let struct_construction = generate_struct_construction(fields);

    let expanded = quote! {
        impl #struct_name {
            pub fn builtin_fn(
                ctx: &mut ::pine_interpreter::Interpreter,
                call_args: ::pine_interpreter::FunctionCallArgs,
            ) -> Result<::pine_interpreter::Value, ::pine_interpreter::RuntimeError> {
                use ::pine_interpreter::{Value, RuntimeError, EvaluatedArg};

                // Extract type parameters first
                #type_param_extraction

                let args = call_args.args;

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

fn parse_builtin_attributes(input: &DeriveInput) -> (String, usize) {
    let mut function_name = None;
    let mut type_params_count = 0;

    for attr in &input.attrs {
        if let Meta::List(meta_list) = &attr.meta {
            if meta_list.path.is_ident("builtin") {
                let tokens_str = meta_list.tokens.to_string();

                // Parse name
                if let Some(start) = tokens_str.find('"') {
                    if let Some(end) = tokens_str[start + 1..].find('"') {
                        function_name = Some(tokens_str[start + 1..start + 1 + end].to_string());
                    }
                }

                // Parse type_params
                if let Some(type_params_pos) = tokens_str.find("type_params") {
                    let after_eq = &tokens_str[type_params_pos..];
                    if let Some(eq_pos) = after_eq.find('=') {
                        let num_str = after_eq[eq_pos + 1..].trim();
                        // Extract just the number (could be followed by comma or end)
                        let num_str = num_str.split(',').next().unwrap_or(num_str).trim();
                        if let Ok(count) = num_str.parse::<usize>() {
                            type_params_count = count;
                        }
                    }
                }
            }
        }
    }

    let function_name =
        function_name.expect("BuiltinFunction requires a #[builtin(name = \"...\")] attribute");
    (function_name, type_params_count)
}

fn generate_type_param_extraction(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
    expected_count: usize,
) -> proc_macro2::TokenStream {
    // Find fields marked with #[type_param]
    let type_param_fields: Vec<_> = fields.iter().filter(|f| is_field_type_param(f)).collect();

    if type_param_fields.is_empty() {
        panic!(
            "Expected {} type parameter fields marked with #[type_param], but found none",
            expected_count
        );
    }

    if type_param_fields.len() != expected_count {
        panic!(
            "Expected {} type parameter fields, but found {}",
            expected_count,
            type_param_fields.len()
        );
    }

    let mut extractions = Vec::new();
    for (idx, field) in type_param_fields.iter().enumerate() {
        let field_name = field.ident.as_ref().unwrap();
        extractions.push(quote! {
            let #field_name = call_args.type_args.get(#idx)
                .ok_or_else(|| RuntimeError::TypeError(
                    format!("Missing type parameter {} (expected {} type parameters)", #idx, #expected_count)
                ))?
                .clone();
        });
    }

    quote! {
        #(#extractions)*
    }
}

fn is_field_type_param(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        if let Meta::Path(path) = &attr.meta {
            path.is_ident("type_param")
        } else {
            false
        }
    })
}

fn generate_field_parsing(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let mut field_decls = Vec::new();
    let mut positional_matches = Vec::new();
    let mut named_matches = Vec::new();
    let mut variadic_field: Option<&syn::Ident> = None;
    let mut non_variadic_count = 0;
    let mut arg_position = 0; // Track positional argument index (excluding type params)

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type = &field.ty;

        // Skip type parameter fields - they're extracted separately
        if is_field_type_param(field) {
            continue;
        }

        // Check if this is a variadic field
        let is_variadic = is_field_variadic(field);

        if is_variadic {
            // Variadic field should be Vec<Value>
            field_decls.push(quote! {
                let mut #field_name: Vec<Value> = Vec::new();
            });
            variadic_field = Some(field_name);
            continue;
        }

        non_variadic_count += 1;

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

        let current_position = arg_position;
        positional_matches.push(quote! {
            #current_position => { #positional_assign }
        });
        arg_position += 1;

        // Generate named assignment
        let named_assign = generate_value_conversion(field_name, field_type, has_default);

        named_matches.push(quote! {
            #field_name_str => { #named_assign }
        });
    }

    // Generate the argument parsing loop
    let arg_parsing = if let Some(variadic_name) = variadic_field {
        quote! {
            let mut positional_idx = 0;
            for arg in args {
                match arg {
                    EvaluatedArg::Positional(arg_value) => {
                        if positional_idx < #non_variadic_count {
                            match positional_idx {
                                #(#positional_matches)*
                                _ => {}
                            }
                        } else {
                            // Collect remaining args as variadic
                            #variadic_name.push(arg_value);
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
    } else {
        quote! {
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
    };

    quote! {
        #(#field_decls)*

        // Validate no duplicate named arguments
        {
            let mut seen_names = std::collections::HashSet::new();
            for arg in &args {
                if let EvaluatedArg::Named { name, .. } = arg {
                    if !seen_names.insert(name.clone()) {
                        return Err(RuntimeError::TypeError(format!("Duplicate argument: {}", name)));
                    }
                }
            }
        }

        #arg_parsing
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
                            let default_tokens: proc_macro2::TokenStream =
                                default_str.parse().unwrap();
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

fn is_field_variadic(field: &Field) -> bool {
    for attr in &field.attrs {
        if let Meta::List(meta_list) = &attr.meta {
            if meta_list.path.is_ident("arg") {
                let tokens_str = meta_list.tokens.to_string();
                if tokens_str.contains("variadic") {
                    return true;
                }
            }
        }
    }
    false
}

fn generate_value_conversion(
    field_name: &syn::Ident,
    field_type: &syn::Type,
    has_default: bool,
) -> proc_macro2::TokenStream {
    let type_str = quote! { #field_type }.to_string();

    // Check if this is an Option type
    let is_option = type_str.contains("Option");

    let conversion = if is_option {
        // For Option types, handle Na values
        // Check for Color BEFORE other types since Value might also be in the type string
        if type_str.contains("Color") && !type_str.contains("Value") {
            quote! {
                if matches!(arg_value, Value::Na) {
                    None
                } else {
                    Some(arg_value.as_color()?)
                }
            }
        } else if type_str.contains("f64") {
            quote! {
                if matches!(arg_value, Value::Na) {
                    None
                } else {
                    Some(arg_value.as_number()?)
                }
            }
        } else if type_str.contains("String") {
            quote! {
                if matches!(arg_value, Value::Na) {
                    None
                } else {
                    Some(arg_value.as_string()?)
                }
            }
        } else if type_str.contains("bool") {
            quote! {
                if matches!(arg_value, Value::Na) {
                    None
                } else {
                    Some(arg_value.as_bool()?)
                }
            }
        } else if type_str.contains("Value") {
            quote! {
                if matches!(arg_value, Value::Na) {
                    None
                } else {
                    Some(arg_value)
                }
            }
        } else {
            quote! {
                if matches!(arg_value, Value::Na) {
                    None
                } else {
                    Some(arg_value)
                }
            }
        }
    } else if type_str.contains("f64") {
        quote! { arg_value.as_number()? }
    } else if type_str.contains("String") {
        quote! { arg_value.as_string()? }
    } else if type_str.contains("bool") {
        quote! { arg_value.as_bool()? }
    } else if type_str.contains("Color") {
        quote! { arg_value.as_color()? }
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

fn generate_field_validation(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let mut validations = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        // Skip type parameter fields - they're validated separately
        if is_field_type_param(field) {
            continue;
        }

        // Skip variadic fields - they don't need validation
        if is_field_variadic(field) {
            continue;
        }

        // Check if field has a default
        let has_default = field.attrs.iter().any(|attr| {
            if let Meta::List(meta_list) = &attr.meta {
                meta_list.path.is_ident("arg") && meta_list.tokens.to_string().contains("default")
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

fn generate_struct_construction(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let field_assignments: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            quote! { #field_name }
        })
        .collect();

    quote! {
        #(#field_assignments),*
    }
}
