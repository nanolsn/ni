use common::{
    OpType,
    UWord,
};

use super::{
    Layout,
    Field,
    Ty,
};

#[derive(Debug, Eq, PartialEq)]
pub enum LayoutError {
    UnexpectedIndirection,
}

pub struct LayoutBuilder<'n> {
    blocks: Vec<Block<'n>>,
}

impl<'n> LayoutBuilder<'n> {
    pub fn new() -> Self { Self { blocks: vec![] } }

    pub fn new_op_type(&mut self, name: &'n str, op: OpType) {
        self.blocks.push(Block::New { ty: BlockType::OpType(op), name })
    }

    pub fn new_layout(&mut self, name: &'n str, lay_idx: usize) {
        self.blocks.push(Block::New { ty: BlockType::Layout(lay_idx), name })
    }

    pub fn new_fn(&mut self, name: &'n str) {
        self.blocks.push(Block::New { ty: BlockType::Function, name })
    }

    pub fn add_indirect(&mut self) { self.blocks.push(Block::Indirect) }

    pub fn add_array(&mut self, size: UWord) { self.blocks.push(Block::Array(size)) }

    pub fn build<'t>(self) -> Result<Layout<'n, 't>, LayoutError> {
        let (n_fields, n_types) = self.blocks
            .iter()
            .fold((0usize, 0usize), |(fx, ix), b| match b {
                Block::New { .. } => (fx + 1, ix),
                Block::Array(_) => (fx, ix + 1),
                Block::Indirect => (fx, ix + 1),
            });

        let mut fields = Vec::with_capacity(n_fields);
        let mut types = Vec::with_capacity(n_types);

        for block in self.blocks {
            if let Block::New { ty, name } = block {
                fields.push(Field {
                    name,
                    ty: ty.into_ty(),
                    ptr: 0,
                });
            } else {
                let field = fields
                    .last_mut()
                    .ok_or(LayoutError::UnexpectedIndirection)?;

                types.push(field.ty);
                let ty = unsafe {
                    let ty = types.last().unwrap();
                    // It’s safe to share this reference since:
                    //
                    // * The reference points to the heap allocated vector.
                    // * The vector will not be reallocated
                    //   as we allocated whole place in advance.
                    // * The allocated memory will not mutate,
                    //   since the final result is placed into immutable `View`.
                    &*(ty as *const Ty)
                };

                field.ty = match block {
                    Block::Array(idx) => Ty::Array(ty, idx),
                    Block::Indirect => Ty::Indirect(ty),
                    _ => unreachable!(),
                };
            }
        }

        Ok(Layout {
            fields: fields.into(),
            types: types.into(),
        })
    }
}

enum Block<'n> {
    New { ty: BlockType, name: &'n str },
    Array(UWord),
    Indirect,
}

enum BlockType {
    OpType(OpType),
    Layout(usize),
    Function,
}

impl BlockType {
    fn into_ty(self) -> Ty<'static> {
        match self {
            BlockType::OpType(op) => Ty::OpType(op),
            BlockType::Layout(idx) => Ty::Layout(idx),
            BlockType::Function => Ty::Function,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_builder() {
        let lay = {
            let mut builder = LayoutBuilder::new();
            builder.new_layout("lay", 0);
            builder.add_indirect();
            builder.add_array(4);
            builder.build().unwrap()
        };

        assert!(matches!(lay.fields[0].ty, Ty::Array(Ty::Indirect(Ty::Layout(0)), 4)));

        let lay = {
            let mut builder = LayoutBuilder::new();
            builder.new_layout("lay", 0);
            builder.add_array(2);
            builder.add_array(3);
            builder.build().unwrap()
        };

        assert!(matches!(lay.fields[0].ty, Ty::Array(Ty::Array(Ty::Layout(0), 2), 3)));
    }

    #[test]
    fn layout_error() {
        let res = {
            let mut builder = LayoutBuilder::new();
            builder.add_indirect();
            builder.build().unwrap_err()
        };

        assert_eq!(res, LayoutError::UnexpectedIndirection);
    }
}
