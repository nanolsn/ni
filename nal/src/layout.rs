use common::{
    OpType,
    UWord,
};

use super::{
    WORD_SIZE,
    view::View,
};

#[derive(Debug)]
pub struct Layout<'n, 't> {
    pub(super) fields: View<Field<'n, 't>>,
    pub(super) types: View<Ty<'t>>,
}

#[derive(Copy, Clone, Debug)]
pub struct Field<'n, 't> {
    pub(super) name: &'n str,
    pub(super) ty: Ty<'t>,
    pub(super) ptr: UWord,
}

#[derive(Copy, Clone, Debug)]
pub enum Ty<'t> {
    OpType(OpType),
    Layout(usize),
    Array(&'t Ty<'t>, UWord),
    Indirect(&'t Ty<'t>),
    Function,
    Undefined,
}
