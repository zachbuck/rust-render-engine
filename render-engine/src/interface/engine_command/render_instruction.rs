
use uuid::Uuid;

#[derive(Debug)]
pub enum RenderInstruction {
	BeginRendering {
		uuid: Uuid,
	},
	EndRendering,

	RenderObject {
		uuid: Uuid,
	}
}