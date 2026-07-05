//! AST-to-AST transformation infrastructure and the [`Transform`] trait.

pub mod passes;

use crate::ast::expr::*;
use crate::ast::lit::*;
use crate::ast::pat::*;
use crate::ast::program::Program;
use crate::ast::stmt::*;

pub trait Transform {
    fn name(&self) -> &'static str;
    fn apply(&self, program: &mut Program);
}

pub fn run_transforms(program: &mut Program, transforms: &[Box<dyn Transform>]) {
    for transform in transforms {
        transform.apply(program);
    }
}
