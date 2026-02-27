/// OOP infrastructure for Axiom — VTable, Class, Instance
/// Supports cls, ext (inheritance), fun init (constructor), method dispatch

use crate::ast::{Stmt, Expr};
use dashmap::DashMap;
use std::sync::Arc;
use std::fmt;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// AxCallable — user-defined or native function
// ---------------------------------------------------------------------------
#[derive(Clone)]
pub enum AxCallable {
    UserDefined {
        params: Vec<String>,
        body: Vec<Stmt>,
        /// Captured lexical environment (closure variables)
        captured: HashMap<String, crate::core::value::AxValue>,
    },
    Native {
        name: String,
        func: fn(Vec<crate::core::value::AxValue>) -> crate::core::value::AxValue,
    },
}

impl fmt::Debug for AxCallable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AxCallable::UserDefined { params, .. } => {
                write!(f, "AxCallable::UserDefined(params={:?})", params)
            }
            AxCallable::Native { name, .. } => {
                write!(f, "AxCallable::Native({})", name)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AxClass — name, parent for ext, methods, constructor
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct AxClass {
    pub name: String,
    pub parent: Option<Arc<AxClass>>,
    pub methods: HashMap<String, AxCallable>,
    pub fields: Vec<(String, Option<Expr>)>,
}

impl AxClass {
    pub fn new(name: String) -> Self {
        AxClass {
            name,
            parent: None,
            methods: HashMap::new(),
            fields: Vec::new(),
        }
    }

    /// VTable dispatch: look up method by name, traversing parent chain.
    pub fn resolve_method(&self, method_name: &str) -> Option<&AxCallable> {
        if let Some(m) = self.methods.get(method_name) {
            return Some(m);
        }
        if let Some(ref parent) = self.parent {
            return parent.resolve_method(method_name);
        }
        None
    }

    pub fn has_init(&self) -> bool {
        self.methods.contains_key("init")
    }
}

// ---------------------------------------------------------------------------
// AxInstance — runtime instance of a class
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct AxInstance {
    pub class: Arc<AxClass>,
    pub fields: DashMap<String, crate::core::value::AxValue>,
}

impl AxInstance {
    pub fn new(class: Arc<AxClass>) -> Self {
        let fields = DashMap::new();
        // Initialize default fields from class hierarchy
        Self::init_fields_from_class(&class, &fields);
        AxInstance { class, fields }
    }

    fn init_fields_from_class(
        class: &AxClass,
        fields: &DashMap<String, crate::core::value::AxValue>,
    ) {
        // Initialize parent fields first
        if let Some(ref parent) = class.parent {
            Self::init_fields_from_class(parent, fields);
        }
        // Then own fields (overriding parent defaults)
        for (name, _default) in &class.fields {
            if !fields.contains_key(name) {
                fields.insert(name.clone(), crate::core::value::AxValue::Nil);
            }
        }
    }

    pub fn get_field(&self, name: &str) -> Option<crate::core::value::AxValue> {
        self.fields.get(name).map(|v| v.clone())
    }

    pub fn set_field(&self, name: &str, value: crate::core::value::AxValue) {
        self.fields.insert(name.to_string(), value);
    }

    pub fn resolve_method(&self, name: &str) -> Option<AxCallable> {
        self.class.resolve_method(name).cloned()
    }
}

impl Clone for AxInstance {
    fn clone(&self) -> Self {
        let fields = DashMap::new();
        for entry in self.fields.iter() {
            fields.insert(entry.key().clone(), entry.value().clone());
        }
        AxInstance {
            class: Arc::clone(&self.class),
            fields,
        }
    }
}

// ---------------------------------------------------------------------------
// AxEnum — runtime enum definition
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct AxEnum {
    pub name: String,
    pub variants: Vec<AxEnumVariantDef>,
}

#[derive(Debug, Clone)]
pub struct AxEnumVariantDef {
    pub name: String,
    pub has_data: bool,
}
