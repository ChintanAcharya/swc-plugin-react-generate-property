#![allow(dead_code)]
#![allow(unused_imports)]

mod options;

use std::ffi::OsStr;
use std::path::{Component, Path};

use options::PluginOptions;

use serde::{Deserialize, Serialize};

use swc_core::ecma::{
    ast::{JSXAttrName, JSXAttrOrSpread::JSXAttr, JSXElement, Program},
    transforms::testing::test,
    visit::{as_folder, FoldWith, VisitMut, VisitMutWith},
};

use swc_core::plugin::{
    metadata::TransformPluginMetadataContextKind, plugin_transform,
    proxies::TransformPluginProgramMetadata,
};

use swc_ecma_parser::{EsConfig, Syntax};

pub struct TransformVisitor<'a> {
    filename: &'a Path,
    root_dir: &'a Path,
    options: PluginOptions,
    options_raw: Option<String>,
}

impl<'a> VisitMut for TransformVisitor<'a> {
    // Implement necessary visit_mut_* methods for actual custom transform.
    // A comprehensive list of possible visitor methods can be found here:
    // https://rustdoc.swc.rs/swc_ecma_visit/trait.VisitMut.html
    fn visit_mut_program(&mut self, program: &mut Program) {
        let relative_path = self.filename.strip_prefix(self.root_dir).unwrap();
        let mut components: Vec<&OsStr> = Vec::new();

        for component in relative_path.components() {
            match component {
                // Component::Prefix(_) => println!("Prefix: {:?}", component),
                // Component::RootDir => println!("Root directory: {:?}", component),
                // Component::CurDir => println!("Current directory: {:?}", component),
                // Component::ParentDir => println!("Parent directory: {:?}", component),
                Component::Normal(os_str) => components.push(&os_str),
                _ => {}
            }
        }

        program.visit_mut_children_with(&mut JSXVisitor {});

        // eprintln!("Transforming: {:?}", self.filename);
        // eprintln!("Root directory: {:?}", self.root_dir);
        // eprintln!("Relative path: {:?}", relative_path);
        // eprintln!("Components: {:?}", components);
        // eprintln!("Raw options: {:?}", self.options_raw);
    }
}

pub struct JSXVisitor {}

impl VisitMut for JSXVisitor {
    fn visit_mut_jsx_element(&mut self, jsx_path: &mut JSXElement) {
        let opening_path = &mut jsx_path.opening;

        let data_id_defined =
            opening_path
                .attrs
                .iter()
                .any(|attr_or_spread| match attr_or_spread {
                    JSXAttr(attr) => {
                        let name = match &attr.name {
                            JSXAttrName::Ident(ident) => ident.sym.as_str(),
                            _ => "",
                        };
                        name == "data-id"
                    }
                    _ => false,
                });

        eprintln!("_data_id_defined: {:?}", data_id_defined);
    }
}

/// An example plugin function with macro support.
/// `plugin_transform` macro interop pointers into deserialized structs, as well
/// as returning ptr back to host.
///
/// It is possible to opt out from macro by writing transform fn manually
/// if plugin need to handle low-level ptr directly via
/// `__transform_plugin_process_impl(
///     ast_ptr: *const u8, ast_ptr_len: i32,
///     unresolved_mark: u32, should_enable_comments_proxy: i32) ->
///     i32 /*  0 for success, fail otherwise.
///             Note this is only for internal pointer interop result,
///             not actual transform result */`
///
/// This requires manual handling of serialization / deserialization from ptrs.
/// Refer swc_plugin_macro to see how does it work internally.
#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let filename = metadata.get_context(&TransformPluginMetadataContextKind::Filename);
    let filename = if let Some(filename) = filename.as_deref() {
        filename
    } else {
        "/unknown.js"
    };
    let filename = Path::new(filename);

    let root_dir = metadata.get_context(&TransformPluginMetadataContextKind::Cwd);
    let root_dir = if let Some(root_dir) = root_dir.as_deref() {
        root_dir
    } else {
        "/"
    };
    let root_dir = Path::new(root_dir);

    let plugin_config = metadata.get_transform_plugin_config();

    let plugin_options: PluginOptions = if let Some(plugin_config) = plugin_config.as_ref() {
        serde_json::from_str(plugin_config).unwrap_or_else(|f| {
            println!("Could not deserialize plugin options");
            println!("{:#?}", f);
            Default::default()
        })
    } else {
        Default::default()
    };

    program.fold_with(&mut as_folder(TransformVisitor {
        filename,
        root_dir,
        options: plugin_options,
        options_raw: plugin_config,
    }))
}

// An example to test plugin transform.
// Recommended strategy to test plugin's transform is verify
// the Visitor's behavior, instead of trying to run `process_transform` with mocks
// unless explicitly required to do so.
test!(
    Syntax::Es(EsConfig {
        jsx: true,
        ..Default::default()
    }),
    |_| as_folder(TransformVisitor {
        filename: Path::new("/src/client/Components/Common/Header.jsx"),
        options: Default::default(),
        root_dir: Path::new("/"),
        options_raw: Default::default(),
    }),
    basic_test,
    // Input codes
    r#"<View><Logo>...</Logo><Content>...</Content></View>"#
);
