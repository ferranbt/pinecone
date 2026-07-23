use proc_macro::TokenStream;
use quote::{format_ident, quote};
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
#[proc_macro_derive(BuiltinFunction, attributes(builtin, arg, type_param, state))]
pub fn builtin_function_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Parse the function name and type params count from attributes
    let (function_name, type_params_count, output_bound, stateful) =
        parse_builtin_attributes(&input);

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
    let signature_fn = generate_signature(fields);
    let field_parsing = generate_field_parsing(fields);
    let field_validation = generate_field_validation(fields);
    let struct_construction = generate_struct_construction(fields);

    // A builtin is stored as `BuiltinFn<O>`, so the generated `builtin_fn` must be
    // generic over the output type. Where `O` is declared depends on the struct:
    // one holding a `Value<O>` field declares `O` itself — along with any
    // capability bound it needs, e.g. `struct LabelNew<O: PineOutput + LabelOutput>`
    // — so that declaration is forwarded verbatim. A struct with only plain fields
    // (`f64`, `String`) has nothing to parameterize, so `O` goes on the method.
    let (decl_impl_generics, decl_ty_generics, where_clause) = input.generics.split_for_impl();
    let (impl_generics, ty_generics, fn_generics) = if input.generics.params.is_empty() {
        // `#[builtin(output = LabelOutput)]` adds the capability the builtin needs
        // from the output type; without it `PineOutput` alone is enough.
        let capability = output_bound
            .map(|bound| quote! { + ::pine_interpreter::#bound })
            .unwrap_or_default();
        (
            quote! {},
            quote! {},
            quote! { <O: ::pine_interpreter::PineOutput #capability> },
        )
    } else {
        (
            quote! { #decl_impl_generics },
            quote! { #decl_ty_generics },
            quote! {},
        )
    };

    let body = quote! {
        // Extract type parameters first
        #type_param_extraction

        let args = call_args.args;

        #field_parsing

        #field_validation
    };

    let expanded = if stateful {
        let state_fields: Vec<_> = fields.iter().filter(|f| is_field_state(f)).collect();
        let state_names: Vec<_> = state_fields
            .iter()
            .map(|f| f.ident.as_ref().unwrap())
            .collect();
        let state_decls: Vec<_> = state_fields
            .iter()
            .map(|f| {
                let name = f.ident.as_ref().unwrap();
                let ty = &f.ty;
                quote! { #name: #ty }
            })
            .collect();
        let slot_name = format_ident!("{}Slot", struct_name);

        quote! {
            /// One call site's memory for the builtin above: its `#[state]`
            /// fields, plus the bar it last advanced on and what it returned
            /// then, so re-entering a call site within one bar cannot advance
            /// the state twice.
            #[derive(Default)]
            #[doc(hidden)]
            pub struct #slot_name<O: ::pine_interpreter::PineOutput> {
                #(#state_decls,)*
                bar_seq: u64,
                memo: Option<::pine_interpreter::Value<O>>,
            }

            impl #impl_generics #struct_name #ty_generics #where_clause {
                /// Builds the callable. Each returned closure owns the state for
                /// every call site of this builtin, keyed by call id.
                pub fn builtin_fn #fn_generics () -> ::pine_interpreter::BuiltinFn<O> {
                    let slots: ::std::rc::Rc<::std::cell::RefCell<
                        ::std::collections::HashMap<u32, #slot_name<O>>
                    >> = Default::default();

                    ::std::rc::Rc::new(move |
                        ctx: &mut ::pine_interpreter::Interpreter<O>,
                        call_args: ::pine_interpreter::FunctionCallArgs<O>,
                    | -> Result<::pine_interpreter::Value<O>, ::pine_interpreter::RuntimeError> {
                        use ::pine_interpreter::{Value, RuntimeError, EvaluatedArg};

                        let call_id = call_args.call_id;
                        let bar_seq = ctx.bar_seq();

                        // Already ran on this bar: hand back the same value
                        // rather than advancing the state again.
                        {
                            let slots = slots.borrow();
                            if let Some(slot) = slots.get(&call_id) {
                                if slot.bar_seq == bar_seq {
                                    if let Some(memo) = &slot.memo {
                                        return Ok(memo.clone());
                                    }
                                }
                            }
                        }

                        #body

                        let mut instance = Self {
                            #struct_construction
                        };

                        // Restore what this call site accumulated on earlier bars.
                        {
                            let slots = slots.borrow();
                            if let Some(slot) = slots.get(&call_id) {
                                #(instance.#state_names = slot.#state_names.clone();)*
                            }
                        }

                        let result = instance.execute(ctx)?;

                        let mut slots = slots.borrow_mut();
                        let slot = slots.entry(call_id).or_default();
                        #(slot.#state_names = instance.#state_names;)*
                        slot.bar_seq = bar_seq;
                        slot.memo = Some(result.clone());

                        Ok(result)
                    })
                }

                pub fn name() -> &'static str {
                    #function_name
                }

                #signature_fn
                /// The registered value: the callable plus the arguments it
                /// accepts, so a caller registers one thing, not two.
                pub fn builtin_value #fn_generics () -> ::pine_interpreter::Value<O> {
                    ::pine_interpreter::Value::BuiltinFunction(::pine_interpreter::Builtin {
                        call: Self::builtin_fn::<O>(),
                        signature: Self::signature(),
                    })
                }

            }
        }
    } else {
        quote! {
            impl #impl_generics #struct_name #ty_generics #where_clause {
                pub fn builtin_fn #fn_generics (
                    ctx: &mut ::pine_interpreter::Interpreter<O>,
                    call_args: ::pine_interpreter::FunctionCallArgs<O>,
                ) -> Result<::pine_interpreter::Value<O>, ::pine_interpreter::RuntimeError> {
                    use ::pine_interpreter::{Value, RuntimeError, EvaluatedArg};

                    #body

                    let instance = Self {
                        #struct_construction
                    };

                    instance.execute(ctx)
                }

                pub fn name() -> &'static str {
                    #function_name
                }

                #signature_fn
                /// The registered value: the callable plus the arguments it
                /// accepts, so a caller registers one thing, not two.
                pub fn builtin_value #fn_generics () -> ::pine_interpreter::Value<O> {
                    ::pine_interpreter::Value::BuiltinFunction(::pine_interpreter::Builtin {
                        call: ::std::rc::Rc::new(Self::builtin_fn),
                        signature: Self::signature(),
                    })
                }

            }
        }
    };

    TokenStream::from(expanded)
}

/// Parses `#[builtin(name = "...", type_params = N, output = Trait)]`.
///
/// `output` names an extra capability the builtin needs from the output type
/// (`LogOutput`, `PlotOutput`, `LabelOutput`, `BoxOutput`). It only applies to
/// structs that declare no generics of their own — one that holds a `Value<O>`
/// states its bounds on `O` directly.
fn parse_builtin_attributes(input: &DeriveInput) -> (String, usize, Option<syn::Ident>, bool) {
    let mut function_name = None;
    let mut type_params_count = 0;
    let mut output_bound = None;
    let mut stateful = false;

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

                // Parse output capability bound
                if let Some(output_pos) = tokens_str.find("output") {
                    let after_eq = &tokens_str[output_pos..];
                    if let Some(eq_pos) = after_eq.find('=') {
                        let bound = after_eq[eq_pos + 1..]
                            .split(',')
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if !bound.is_empty() {
                            output_bound =
                                Some(syn::Ident::new(&bound, proc_macro2::Span::call_site()));
                        }
                    }
                }

                // `stateful` opts the builtin into per-call-site memory.
                stateful |= tokens_str.contains("stateful");
            }
        }
    }

    let function_name =
        function_name.expect("BuiltinFunction requires a #[builtin(name = \"...\")] attribute");
    (function_name, type_params_count, output_bound, stateful)
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

/// Build the [`BuiltinSignature`] literal describing this builtin's arguments,
/// so semantic analysis can reject a call before it runs. Mirrors the field
/// types, since those are what the runtime conversion will apply.
fn generate_signature(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let params = fields
        .iter()
        // `#[type_param]` and `#[state]` fields are not call arguments.
        .filter(|f| !is_field_type_param(f) && !is_field_state(f))
        .map(|field| {
            let name = field
                .ident
                .as_ref()
                .unwrap()
                .to_string()
                .trim_start_matches("r#")
                .to_string();
            let field_ty = &field.ty;
            let type_str = quote! { #field_ty }.to_string();
            let variadic = is_field_variadic(field);
            let (has_default, _) = parse_field_default(field);
            // A defaulted, variadic or Option field may be omitted.
            let required = !has_default && !variadic && !type_str.contains("Option");

            let ty = if type_str.contains("Value") || type_str.contains("Vec") {
                quote! { ::pine_interpreter::ParamType::Any }
            } else if type_str.contains("Color") {
                quote! { ::pine_interpreter::ParamType::Color }
            } else if type_str.contains("String") {
                quote! { ::pine_interpreter::ParamType::String }
            } else if type_str.contains("bool") {
                quote! { ::pine_interpreter::ParamType::Bool }
            } else if type_str.contains("f64")
                || type_str.contains("i64")
                || type_str.contains("Num")
            {
                quote! { ::pine_interpreter::ParamType::Number }
            } else {
                quote! { ::pine_interpreter::ParamType::Any }
            };

            quote! {
                ::pine_interpreter::Param {
                    name: #name.to_string(),
                    ty: #ty,
                    required: #required,
                    variadic: #variadic,
                }
            }
        });

    quote! {
        pub fn signature() -> ::pine_interpreter::BuiltinSignature {
            ::pine_interpreter::BuiltinSignature {
                params: vec![#(#params),*],
            }
        }
    }
}

/// `#[state]` marks a field that is not a Pine argument but the builtin's own
/// memory, carried across bars by the call site rather than parsed from a call.
fn is_field_state(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        if let Meta::Path(path) = &attr.meta {
            path.is_ident("state")
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
        // A raw identifier like `r#type` names the Pine argument `type`; strip the
        // `r#` so the named-argument key matches what the caller wrote.
        let field_name_str = field_name.to_string().trim_start_matches("r#").to_string();
        let field_type = &field.ty;

        // Skip type parameter fields - they're extracted separately
        if is_field_type_param(field) {
            continue;
        }

        // Skip state fields - they come from the call site, not the arguments
        if is_field_state(field) {
            continue;
        }

        // Check if this is a variadic field
        let is_variadic = is_field_variadic(field);

        if is_variadic {
            // Variadic field is a `Vec<Value<O>>`; take the declared type rather
            // than naming `Value` here, which would not know the output type.
            field_decls.push(quote! {
                let mut #field_name: #field_type = Vec::new();
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
        } else if type_str.contains("Num") {
            quote! {
                if matches!(arg_value, Value::Na) {
                    None
                } else {
                    arg_value.as_num()
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
        } else {
            quote! {
                if matches!(arg_value, Value::Na) {
                    None
                } else {
                    Some(arg_value)
                }
            }
        }
    } else if type_str.contains("Num") {
        // A `Num` field keeps int-vs-float, so the builtin can apply Pine's
        // overload rule; `f64` below deliberately discards it.
        quote! {
            arg_value.as_num().ok_or_else(|| RuntimeError::TypeError(
                "Expected number".to_string()
            ))?
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
        let field_name_str = field_name.to_string().trim_start_matches("r#").to_string();

        // Skip type parameter fields - they're validated separately
        if is_field_type_param(field) {
            continue;
        }

        // Skip state fields - they are never supplied by the caller
        if is_field_state(field) {
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
            // State fields start empty; the call site's saved values are restored
            // over them right after construction.
            if is_field_state(field) {
                quote! { #field_name: Default::default() }
            } else {
                quote! { #field_name }
            }
        })
        .collect();

    quote! {
        #(#field_assignments),*
    }
}
