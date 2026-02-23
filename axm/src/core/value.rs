/// Core value types for Axiom runtime — Final Maturation
/// Supports: Num, Str, Bol, Lst, Map, Obj, Nil, Instance, EnumVariant, Fun

use crate::core::oop::{AxCallable, AxInstance};
use dashmap::DashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub enum ValidationError {
    TypeError(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::TypeError(msg) => write!(f, "TypeError: {}", msg),
        }
    }
}

/// AxObject — simple named struct (legacy, pre-OOP)
#[derive(Debug, Clone)]
pub struct AxObject {
    pub type_name: String,
    pub fields: Arc<DashMap<String, AxValue>>,
    pub methods: Arc<DashMap<String, AxValue>>,
}

impl AxObject {
    pub fn new(type_name: String) -> Self {
        AxObject {
            type_name,
            fields: Arc::new(DashMap::new()),
            methods: Arc::new(DashMap::new()),
        }
    }
}

/// AxValue — the universal runtime value type for Axiom
#[derive(Clone)]
pub enum AxValue {
    Num(f64),
    Str(String),
    Bol(bool),
    Lst(Arc<RwLock<Vec<AxValue>>>),
    Map(Arc<DashMap<String, AxValue>>),
    Obj(AxObject),
    Instance(Arc<RwLock<AxInstance>>),
    EnumVariant(Arc<str>, Box<AxValue>),
    Fun(Arc<AxCallable>),
    Nil,
}

impl fmt::Debug for AxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AxValue::Num(n) => write!(f, "Num({})", n),
            AxValue::Str(s) => write!(f, "Str(\"{}\")", s),
            AxValue::Bol(b) => write!(f, "Bol({})", b),
            AxValue::Lst(_) => write!(f, "Lst([...])"),
            AxValue::Map(_) => write!(f, "Map({{...}})"),
            AxValue::Obj(o) => write!(f, "Obj({})", o.type_name),
            AxValue::Instance(inst) => {
                let i = inst.read().unwrap();
                write!(f, "Instance({})", i.class.name)
            }
            AxValue::EnumVariant(name, val) => write!(f, "EnumVariant({}({:?}))", name, val),
            AxValue::Fun(c) => write!(f, "Fun({:?})", c),
            AxValue::Nil => write!(f, "Nil"),
        }
    }
}

impl AxValue {
    pub fn as_num(&self) -> Result<f64, ValidationError> {
        match self {
            AxValue::Num(n) => Ok(*n),
            _ => Err(ValidationError::TypeError(format!(
                "Expected Num, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn as_str(&self) -> Result<String, ValidationError> {
        match self {
            AxValue::Str(s) => Ok(s.clone()),
            _ => Err(ValidationError::TypeError(format!(
                "Expected Str, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn as_bol(&self) -> Result<bool, ValidationError> {
        match self {
            AxValue::Bol(b) => Ok(*b),
            _ => Err(ValidationError::TypeError(format!(
                "Expected Bol, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn as_lst(&self) -> Result<Arc<RwLock<Vec<AxValue>>>, ValidationError> {
        match self {
            AxValue::Lst(l) => Ok(Arc::clone(l)),
            _ => Err(ValidationError::TypeError(format!(
                "Expected Lst, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn as_map(&self) -> Result<Arc<DashMap<String, AxValue>>, ValidationError> {
        match self {
            AxValue::Map(m) => Ok(Arc::clone(m)),
            _ => Err(ValidationError::TypeError(format!(
                "Expected Map, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn as_obj(&self) -> Result<&AxObject, ValidationError> {
        match self {
            AxValue::Obj(o) => Ok(o),
            _ => Err(ValidationError::TypeError(format!(
                "Expected Obj, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            AxValue::Num(n) => *n != 0.0,
            AxValue::Str(s) => !s.is_empty(),
            AxValue::Bol(b) => *b,
            AxValue::Lst(l) => !l.read().unwrap().is_empty(),
            AxValue::Map(m) => !m.is_empty(),
            AxValue::Nil => false,
            AxValue::Instance(_) => true,
            AxValue::EnumVariant(_, _) => true,
            AxValue::Fun(_) => true,
            AxValue::Obj(_) => true,
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            AxValue::Num(_) => "Num",
            AxValue::Str(_) => "Str",
            AxValue::Bol(_) => "Bol",
            AxValue::Lst(_) => "Lst",
            AxValue::Map(_) => "Map",
            AxValue::Obj(o) => &o.type_name,
            AxValue::Instance(_inst) => {
                // Cannot borrow, return static str
                "Instance"
            }
            AxValue::EnumVariant(_, _) => "EnumVariant",
            AxValue::Fun(_) => "Fun",
            AxValue::Nil => "Nil",
        }
    }

    pub fn display(&self) -> String {
        match self {
            AxValue::Num(n) => {
                if *n == n.floor() && n.is_finite() {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            AxValue::Str(s) => s.clone(),
            AxValue::Bol(b) => format!("{}", b),
            AxValue::Lst(l) => {
                let items = l.read().unwrap();
                let parts: Vec<String> = items.iter().map(|v| v.display()).collect();
                format!("[{}]", parts.join(", "))
            }
            AxValue::Map(m) => {
                let entries: Vec<String> = m
                    .iter()
                    .map(|e| format!("{}: {}", e.key(), e.value().display()))
                    .collect();
                format!("{{{}}}", entries.join(", "))
            }
            AxValue::Obj(o) => format!("<{}>", o.type_name),
            AxValue::Instance(inst) => {
                let i = inst.read().unwrap();
                let fields: Vec<String> = i
                    .fields
                    .iter()
                    .map(|e| format!("{}: {}", e.key(), e.value().display()))
                    .collect();
                format!("<{} {{{}}}>", i.class.name, fields.join(", "))
            }
            AxValue::EnumVariant(name, val) => match val.as_ref() {
                AxValue::Nil => format!("{}", name),
                other => format!("{}({})", name, other.display()),
            },
            AxValue::Fun(_) => "<fun>".to_string(),
            AxValue::Nil => "nil".to_string(),
        }
    }
}
