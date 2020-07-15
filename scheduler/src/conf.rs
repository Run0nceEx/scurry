
use std::error::Error;
use std::sync::RwLock;
use config::Config;

lazy_static! {
	static ref SETTINGS: RwLock<Config> = RwLock::new(Config::default());
}


fn try_main() -> Result<(), Box<Error>> {
	// Set property
	SETTINGS.write()?.set("property", 42)?;

	// Get property
	println!("property: {}", SETTINGS.read()?.get::<i32>("property")?);

	Ok(())
}