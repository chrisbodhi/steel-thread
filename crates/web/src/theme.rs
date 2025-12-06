#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Theme {
    Brighton,
    Industrial,
    Classic,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Brighton => "brighton",
            Theme::Industrial => "industrial",
            Theme::Classic => "classic",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Theme::Brighton => "Brighton",
            Theme::Industrial => "Industrial",
            Theme::Classic => "Classic",
        }
    }

    pub fn all() -> [Theme; 3] {
        [Theme::Brighton, Theme::Industrial, Theme::Classic]
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Brighton
    }
}
