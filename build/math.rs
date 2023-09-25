use quote::{quote, ToTokens};

pub struct LookupTable {
    table: Vec<i16>,
}

impl ToTokens for LookupTable {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fixed_point_literals: Vec<String> = self
            .table
            .iter()
            .map(|i| format!("gba::fixed::i16fx8::from_raw({})", i))
            .collect();

        let array_literal = fixed_point_literals.join(",");
        let array_literal = format!("[ {} ]", array_literal);

        let expr: syn::Expr =
            syn::parse_str(&array_literal).expect("Error producing lookup_table.");

        expr.to_tokens(tokens);
    }
}

fn generate_sine_lookup() -> LookupTable {
    let table: Vec<i16> = (0_u16..512_u16)
        .map(|i| {
            let i: f32 = i.into();
            let radians = (i / 511.0) * (2.0 * std::f32::consts::PI);
            let sin = radians.sin();

            (sin * 2.0_f32.powi(8)) as i16
        })
        .collect();

    LookupTable { table }
}

pub fn generate_lookup_table_src() -> String {
    let sine_lookup = generate_sine_lookup();

    quote! {
        static SINE_LOOKUP: [gba::fixed::i16fx8; 512] = #sine_lookup;
    }
    .to_string()
}
