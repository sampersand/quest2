use crate::Value;

pub trait QuestValue {
	fn parents(&self) -> &[Value];
}

#[repr(C)]
pub struct External {

}
