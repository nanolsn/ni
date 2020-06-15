use common::{
    OpType,
    UWord,
};

const WORD_SIZE: UWord = std::mem::size_of::<UWord>() as UWord;

pub struct Layout<'n, 'l> {
    fields: Vec<Field<'n, 'l>>,
    ret_field: Option<Field<'static, 'l>>,
}

impl<'n, 'l> Layout<'n, 'l> {
    pub fn new(fields: Vec<Field<'n, 'l>>) -> Self {
        Layout {
            fields,
            ret_field: None,
        }
    }

    pub fn set_ret_field(&mut self, field: Field<'static, 'l>) { self.ret_field = Some(field) }

    pub fn size(&self) -> UWord { self.fields.iter().map(|f| f.size() * f.len()).sum() }
}

pub enum Kind<'n, 'l> {
    OpType(OpType),
    Layout(&'l Layout<'n, 'l>),
    Function,
}

pub struct Field<'n, 'l> {
    pub name: &'n str,
    pub kind: Kind<'n, 'l>,
    pub ptr: UWord,
    pub indirections: u32,
    pub array_len: u32,
}

impl Field<'_, '_> {
    pub fn len(&self) -> UWord { self.array_len as UWord }

    pub fn size(&self) -> UWord {
        if self.indirections > 0 {
            WORD_SIZE
        } else {
            match &self.kind {
                Kind::OpType(ty) => ty.size(),
                Kind::Layout(lay) => lay.size(),
                Kind::Function => WORD_SIZE,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout() {
        let fields = vec![
            Field {
                name: "x",
                kind: Kind::OpType(OpType::I32),
                ptr: 0,
                indirections: 1,
                array_len: 1,
            },
            Field {
                name: "y",
                kind: Kind::OpType(OpType::U32),
                ptr: 0,
                indirections: 0,
                array_len: 1,
            },
            Field {
                name: "xs",
                kind: Kind::OpType(OpType::U8),
                ptr: 0,
                indirections: 0,
                array_len: 12,
            },
        ];

        let lay = Layout::new(fields);
        assert_eq!(lay.size(), WORD_SIZE + 4 + 12);
    }
}
