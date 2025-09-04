const COMMANDS: &[&str] = &[];

fn main() {
	#[cfg(not(docsrs))]
	tauri_plugin::Builder::new(COMMANDS)
		.android_path("android")
		.build();
}