use crate::instruction::Instruction;

use std::fmt::{Display, Formatter, Result};
use std::collections::HashMap;
use std::cmp::Ordering;

/// The base object for all Babalang objects
#[derive(Clone, PartialEq, Debug)]
pub struct Object {
    pub reference_count: usize,
    pub obj_type: Type
}

/// The type of a Babalang object
#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Empty(Empty),
    Reference(Reference),
    You(You),
    Group(Group),
    Level(Level),
    Image(Image),
    ImageInstance(ImageInstance)
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Empty {}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Reference {
    // virtual pointer
    pub pointer: usize
}

#[derive(Clone, Copy, Debug)]
pub struct You {
    pub x: u8,
    pub y: u8,
    pub dir: u8 // only lowest 2 bits are used
}

impl PartialEq for You {
    fn eq(&self, other: &You) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl PartialOrd for You {
    fn partial_cmp(&self, other: &You) -> Option<Ordering> {
        match self.dir {
            0 => {
                self.x.partial_cmp(&other.x)
            },
            1 => {
                self.y.partial_cmp(&other.y)
            },
            2 => {
                other.x.partial_cmp(&self.x)
            },
            3 => {
                other.y.partial_cmp(&self.y)
            },
            _ => {
                None
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Group {
    pub index: usize,
    pub data: Vec<Object>
}

impl PartialEq for Group {
    fn eq(&self, other: &Group) -> bool {
        self.data == other.data
    }
}

impl PartialOrd for Group {
    fn partial_cmp(&self, other: &Group) -> Option<Ordering> {
        self.data.len().partial_cmp(&other.data.len())
    }
}

#[derive(Clone, Debug)]
pub struct Level {
    pub identifier: usize,
    pub arguments: Vec<usize>,
    pub parameters: Vec<Object>,
    pub callback: Vec<Instruction>
}

impl PartialEq for Level {
    fn eq(&self, other: &Level) -> bool {
        self.arguments == other.arguments && self.callback == other.callback
    }
}

#[derive(Clone, Debug)]
pub struct Image {
    pub identifier: usize,
    pub constructor: Level,
    pub attributes: HashMap<usize, Option<Object>>,
    pub attribute_pointer: usize
}

impl PartialEq for Image {
    fn eq(&self, other: &Image) -> bool {
        self.constructor == other.constructor && self.attributes == other.attributes
    }
}

#[derive(Clone, Debug)]
pub struct ImageInstance {
    pub class: usize,
    pub attributes: HashMap<usize, Option<Object>>,
    pub attribute_pointer: usize
}

impl PartialEq for ImageInstance {
    fn eq(&self, other: &ImageInstance) -> bool {
        self.class == other.class && self.attributes == other.attributes
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Type::You(_) => write!(f, "YOU"),
            Type::Empty(_) => write!(f, "EMPTY"),
            Type::Group(_) => write!(f, "GROUP"),
            Type::Reference(_) => write!(f, "[REFERENCE]"),
            Type::Level(_) => write!(f, "LEVEL"),
            Type::Image(_) => write!(f, "IMAGE"),
            Type::ImageInstance(_) => write!(f, "[IMAGE INSTANCE]"),
        }
    }
}

pub const EMPTY: Object = Object {
    reference_count: 0,
    obj_type: Type::Empty(Empty {})
};

pub const LEVEL: Object = Object {
    reference_count: 0,
    obj_type: Type::Level(Level {
        identifier: 1,
        arguments: Vec::new(),
        parameters: Vec::new(),
        callback: Vec::new()
    })
};