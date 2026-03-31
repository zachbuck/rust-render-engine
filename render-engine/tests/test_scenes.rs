
macro_rules! render_scene_test {
	($scene: ident) => {
		mod $scene;

		#[test]
		fn $scene() {
			$scene::render_scene();
		}
	};
}

render_scene_test!(basic_triangle);
render_scene_test!(texture_quad);
render_scene_test!(rotated_triangle);
