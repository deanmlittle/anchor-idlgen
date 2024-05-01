use convert_case::{Case, Casing};

use crate::{generators::{cpi::{make_cpi_accounts, make_cpi_ctxs}, i11n::make_i11n_ctxs}, types::Instruction, Sdk, IDL};

pub fn make_defined_types(idl: &IDL) -> String {
    idl.types.iter().map(|t| {
        let ty = if t.kind.kind == "enum" {
    format!("#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum {} {{
{}
}}", t.name, t.kind.variants.clone().unwrap_or(vec![]).iter().map(|n| format!("    {}", n.name.to_case(Case::Pascal))).collect::<Vec<String>>().join(",\n"))
        } else if t.kind.kind == "struct" {
            format!("#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct {} {{
{}
}}", t.name, t.kind.fields.clone().unwrap_or(vec![]).iter().map(|f| {
        format!("    pub {}: {},", f.name.to_case(Case::Snake), f.kind.to_string())
    }).collect::<Vec<String>>().join("\n"))
        } else {
            panic!("Unknown defined type: {}", t.kind.kind);
        };
        ty
    }).collect::<Vec<String>>().join("\n\n")
}

pub fn make_ixs(idl: &IDL) -> String {
    format!("pub mod instructions {{
    use anchor_lang::prelude::*;
    use anchor_i11n::prelude::*;
    use super::*;

{}        
}}",
        idl.instructions.iter().map(|ix| {
        let ix_name_pascal =  ix.name.to_case(Case::Pascal);
        format!("    #[derive(AnchorDiscriminator, AnchorSerialize, AnchorDeserialize)]
    pub struct {} {{
{}
    }}", 
    ix_name_pascal, 
    ix.args.iter().map(|a| format!("        pub {}: {},", a.name.to_case(Case::Snake), a.kind.to_string())).collect::<Vec<String>>().join("\n"))
        }).collect::<Vec<String>>().join("\n\n")
    )
}

pub fn make_ix_args(ix: &Instruction) -> String {
    if ix.args.len() > 0 {
        format!(",\n{}", ix.args.iter().map(|a| format!("        {}: {}", a.name.to_case(Case::Snake), a.kind.to_string())).collect::<Vec<String>>().join(",\n"))
    } else {
        String::new()
    }
}

pub fn make_ix_arg_names(ix: &Instruction) -> String {
    ix.args.iter().map(|a| a.name.to_case(Case::Snake)).collect::<Vec<String>>().join(", ")
}

pub fn make_ix_has_info(ix: &Instruction) -> String {
    match ix.accounts.len() == 0 {
        true => String::new(),
        false => "<'info>".to_string()
    }
}

pub fn make_ix_has_info_colon2(ix: &Instruction) -> String {
    match ix.accounts.len() == 0 {
        true => String::new(),
        false => "::<'info>".to_string()
    }
}

pub fn make_cargo_toml(idl: &IDL, sdk: &Sdk) -> String {
    let i11n = match sdk {
        Sdk::I11n | Sdk::Full => "\nanchor-i11n = { git = \"https://github.com/deanmlittle/anchor-i11n.git\" }".to_string(),
        Sdk::CPI => String::new()
    };
    format!("[package]
name = \"{}-sdk\"
version = \"{}\"
description = \"Created with Anchor-IDLGen\"
edition = \"2021\"

[lib]
crate-type = [\"cdylib\", \"lib\"]
name = \"{}_sdk\"

[dependencies]
anchor-lang = \"0.30.0\"{}", idl.name.to_case(Case::Kebab), idl.version, idl.name.to_case(Case::Snake), i11n)
}

pub fn make_lib_rs(idl: &IDL, sdk: &Sdk) -> String {
    let cpi = match sdk {
        &Sdk::CPI | &Sdk::Full => format!("
// Accounts
{}

// CPI
{}
", make_cpi_accounts(idl), make_cpi_ctxs(idl)),
        &Sdk::I11n => String::new()
    };
    let i11n = match sdk {
        &Sdk::I11n | &Sdk::Full => format!("
// I11n
{}
", make_i11n_ctxs(idl)),
        &Sdk::CPI => String::new()
    };
    let ixs = make_ixs(idl);
    let defined_types = make_defined_types(idl);

    format!("use anchor_lang::prelude::*;

declare_id!(\"{}\");
{}{}
// Instructions
{}
        
// Defined types
{}", idl.metadata.address, cpi, i11n, ixs, defined_types)
}