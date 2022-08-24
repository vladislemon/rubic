use config::Config;

pub struct Settings {
    title: String,
    width: u32,
    height: u32,
    vsync: bool
}

impl Settings {
    pub fn new(path: &str) -> Self {
        let conf = Config::builder()
            .add_source(config::File::with_name(path))
            //.add_source(config::Environment::with_prefix("APP"))
            .build()
            .expect("Failed to build settings");
        let title = conf.get_string("title").unwrap_or("untitled".to_string());
        let width = conf.get_int("width").unwrap_or(1280) as u32;
        let height = conf.get_int("height").unwrap_or(720) as u32;
        let vsync = conf.get_bool("vsync").unwrap_or(true);
        Self {
            title,
            width,
            height,
            vsync
        }
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_vsync(&self) -> bool {
        self.vsync
    }
}
