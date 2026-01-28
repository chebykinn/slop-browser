use super::selector::Selector;
use crate::render::painter::Color;

#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    pub value: Value,
    pub important: bool,
}

#[derive(Debug, Clone)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    Color(Color),
    Number(f32),
    Percentage(f32),
    Auto,
    None,
    /// Multiple values for shorthand properties (e.g., margin: 10px 20px)
    List(Vec<Value>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Px,
    Em,
    Rem,
    Percent,
    Vh,
    Vw,
}

impl Value {
    pub fn to_px(&self, parent_font_size: f32, viewport_width: f32, viewport_height: f32) -> Option<f32> {
        match self {
            Value::Length(v, unit) => {
                let px = match unit {
                    Unit::Px => *v,
                    Unit::Em => *v * parent_font_size,
                    Unit::Rem => *v * 16.0,
                    Unit::Percent => return None,
                    Unit::Vh => *v * viewport_height / 100.0,
                    Unit::Vw => *v * viewport_width / 100.0,
                };
                Some(px)
            }
            Value::Number(v) => Some(*v),
            Value::Percentage(p) => Some(*p),
            _ => None,
        }
    }

    pub fn to_color(&self) -> Option<Color> {
        match self {
            Value::Color(c) => Some(*c),
            Value::Keyword(kw) => match kw.to_lowercase().as_str() {
                // Basic colors
                "black" => Some(Color::BLACK),
                "white" => Some(Color::WHITE),
                "red" => Some(Color::rgb(255, 0, 0)),
                "green" => Some(Color::rgb(0, 128, 0)),
                "blue" => Some(Color::rgb(0, 0, 255)),
                "yellow" => Some(Color::rgb(255, 255, 0)),
                "cyan" | "aqua" => Some(Color::rgb(0, 255, 255)),
                "magenta" | "fuchsia" => Some(Color::rgb(255, 0, 255)),
                "transparent" => Some(Color::TRANSPARENT),

                // Grays
                "gray" | "grey" => Some(Color::rgb(128, 128, 128)),
                "darkgray" | "darkgrey" => Some(Color::rgb(169, 169, 169)),
                "lightgray" | "lightgrey" => Some(Color::rgb(211, 211, 211)),
                "dimgray" | "dimgrey" => Some(Color::rgb(105, 105, 105)),
                "silver" => Some(Color::rgb(192, 192, 192)),
                "gainsboro" => Some(Color::rgb(220, 220, 220)),
                "whitesmoke" => Some(Color::rgb(245, 245, 245)),

                // Reds and pinks
                "maroon" => Some(Color::rgb(128, 0, 0)),
                "darkred" => Some(Color::rgb(139, 0, 0)),
                "firebrick" => Some(Color::rgb(178, 34, 34)),
                "crimson" => Some(Color::rgb(220, 20, 60)),
                "indianred" => Some(Color::rgb(205, 92, 92)),
                "lightcoral" => Some(Color::rgb(240, 128, 128)),
                "salmon" => Some(Color::rgb(250, 128, 114)),
                "lightsalmon" => Some(Color::rgb(255, 160, 122)),
                "coral" => Some(Color::rgb(255, 127, 80)),
                "tomato" => Some(Color::rgb(255, 99, 71)),
                "orangered" => Some(Color::rgb(255, 69, 0)),
                "pink" => Some(Color::rgb(255, 192, 203)),
                "lightpink" => Some(Color::rgb(255, 182, 193)),
                "hotpink" => Some(Color::rgb(255, 105, 180)),
                "deeppink" => Some(Color::rgb(255, 20, 147)),
                "mediumvioletred" => Some(Color::rgb(199, 21, 133)),
                "palevioletred" => Some(Color::rgb(219, 112, 147)),

                // Oranges
                "orange" => Some(Color::rgb(255, 165, 0)),
                "darkorange" => Some(Color::rgb(255, 140, 0)),
                "gold" => Some(Color::rgb(255, 215, 0)),

                // Yellows
                "lightyellow" => Some(Color::rgb(255, 255, 224)),
                "lemonchiffon" => Some(Color::rgb(255, 250, 205)),
                "papayawhip" => Some(Color::rgb(255, 239, 213)),
                "moccasin" => Some(Color::rgb(255, 228, 181)),
                "peachpuff" => Some(Color::rgb(255, 218, 185)),
                "palegoldenrod" => Some(Color::rgb(238, 232, 170)),
                "khaki" => Some(Color::rgb(240, 230, 140)),
                "darkkhaki" => Some(Color::rgb(189, 183, 107)),

                // Purples
                "purple" => Some(Color::rgb(128, 0, 128)),
                "indigo" => Some(Color::rgb(75, 0, 130)),
                "darkmagenta" => Some(Color::rgb(139, 0, 139)),
                "darkviolet" => Some(Color::rgb(148, 0, 211)),
                "darkorchid" => Some(Color::rgb(153, 50, 204)),
                "mediumorchid" => Some(Color::rgb(186, 85, 211)),
                "orchid" => Some(Color::rgb(218, 112, 214)),
                "violet" => Some(Color::rgb(238, 130, 238)),
                "plum" => Some(Color::rgb(221, 160, 221)),
                "lavender" => Some(Color::rgb(230, 230, 250)),
                "thistle" => Some(Color::rgb(216, 191, 216)),
                "blueviolet" => Some(Color::rgb(138, 43, 226)),
                "mediumpurple" => Some(Color::rgb(147, 112, 219)),
                "mediumslateblue" => Some(Color::rgb(123, 104, 238)),
                "slateblue" => Some(Color::rgb(106, 90, 205)),
                "darkslateblue" => Some(Color::rgb(72, 61, 139)),
                "rebeccapurple" => Some(Color::rgb(102, 51, 153)),

                // Blues
                "navy" => Some(Color::rgb(0, 0, 128)),
                "darkblue" => Some(Color::rgb(0, 0, 139)),
                "mediumblue" => Some(Color::rgb(0, 0, 205)),
                "royalblue" => Some(Color::rgb(65, 105, 225)),
                "cornflowerblue" => Some(Color::rgb(100, 149, 237)),
                "lightsteelblue" => Some(Color::rgb(176, 196, 222)),
                "steelblue" => Some(Color::rgb(70, 130, 180)),
                "dodgerblue" => Some(Color::rgb(30, 144, 255)),
                "deepskyblue" => Some(Color::rgb(0, 191, 255)),
                "skyblue" => Some(Color::rgb(135, 206, 235)),
                "lightskyblue" => Some(Color::rgb(135, 206, 250)),
                "lightblue" => Some(Color::rgb(173, 216, 230)),
                "powderblue" => Some(Color::rgb(176, 224, 230)),
                "midnightblue" => Some(Color::rgb(25, 25, 112)),
                "cadetblue" => Some(Color::rgb(95, 158, 160)),
                "slategray" | "slategrey" => Some(Color::rgb(112, 128, 144)),
                "lightslategray" | "lightslategrey" => Some(Color::rgb(119, 136, 153)),
                "aliceblue" => Some(Color::rgb(240, 248, 255)),
                "azure" => Some(Color::rgb(240, 255, 255)),

                // Greens
                "lime" => Some(Color::rgb(0, 255, 0)),
                "limegreen" => Some(Color::rgb(50, 205, 50)),
                "forestgreen" => Some(Color::rgb(34, 139, 34)),
                "darkgreen" => Some(Color::rgb(0, 100, 0)),
                "seagreen" => Some(Color::rgb(46, 139, 87)),
                "mediumseagreen" => Some(Color::rgb(60, 179, 113)),
                "springgreen" => Some(Color::rgb(0, 255, 127)),
                "mediumspringgreen" => Some(Color::rgb(0, 250, 154)),
                "lightgreen" => Some(Color::rgb(144, 238, 144)),
                "palegreen" => Some(Color::rgb(152, 251, 152)),
                "darkseagreen" => Some(Color::rgb(143, 188, 143)),
                "greenyellow" => Some(Color::rgb(173, 255, 47)),
                "chartreuse" => Some(Color::rgb(127, 255, 0)),
                "lawngreen" => Some(Color::rgb(124, 252, 0)),
                "yellowgreen" => Some(Color::rgb(154, 205, 50)),
                "olivedrab" => Some(Color::rgb(107, 142, 35)),
                "olive" => Some(Color::rgb(128, 128, 0)),
                "darkolivegreen" => Some(Color::rgb(85, 107, 47)),
                "mediumaquamarine" => Some(Color::rgb(102, 205, 170)),
                "honeydew" => Some(Color::rgb(240, 255, 240)),
                "mintcream" => Some(Color::rgb(245, 255, 250)),

                // Cyan/Teal
                "teal" => Some(Color::rgb(0, 128, 128)),
                "darkcyan" => Some(Color::rgb(0, 139, 139)),
                "lightcyan" => Some(Color::rgb(224, 255, 255)),
                "aquamarine" => Some(Color::rgb(127, 255, 212)),
                "turquoise" => Some(Color::rgb(64, 224, 208)),
                "mediumturquoise" => Some(Color::rgb(72, 209, 204)),
                "darkturquoise" => Some(Color::rgb(0, 206, 209)),
                "paleturquoise" => Some(Color::rgb(175, 238, 238)),

                // Browns
                "brown" => Some(Color::rgb(165, 42, 42)),
                "saddlebrown" => Some(Color::rgb(139, 69, 19)),
                "sienna" => Some(Color::rgb(160, 82, 45)),
                "chocolate" => Some(Color::rgb(210, 105, 30)),
                "peru" => Some(Color::rgb(205, 133, 63)),
                "sandybrown" => Some(Color::rgb(244, 164, 96)),
                "burlywood" => Some(Color::rgb(222, 184, 135)),
                "tan" => Some(Color::rgb(210, 180, 140)),
                "rosybrown" => Some(Color::rgb(188, 143, 143)),
                "goldenrod" => Some(Color::rgb(218, 165, 32)),
                "darkgoldenrod" => Some(Color::rgb(184, 134, 11)),
                "navajowhite" => Some(Color::rgb(255, 222, 173)),
                "wheat" => Some(Color::rgb(245, 222, 179)),
                "blanchedalmond" => Some(Color::rgb(255, 235, 205)),
                "bisque" => Some(Color::rgb(255, 228, 196)),
                "cornsilk" => Some(Color::rgb(255, 248, 220)),
                "beige" => Some(Color::rgb(245, 245, 220)),
                "antiquewhite" => Some(Color::rgb(250, 235, 215)),
                "linen" => Some(Color::rgb(250, 240, 230)),
                "floralwhite" => Some(Color::rgb(255, 250, 240)),
                "oldlace" => Some(Color::rgb(253, 245, 230)),
                "ivory" => Some(Color::rgb(255, 255, 240)),
                "seashell" => Some(Color::rgb(255, 245, 238)),
                "snow" => Some(Color::rgb(255, 250, 250)),
                "mistyrose" => Some(Color::rgb(255, 228, 225)),
                "lavenderblush" => Some(Color::rgb(255, 240, 245)),
                "ghostwhite" => Some(Color::rgb(248, 248, 255)),

                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_keyword(&self) -> Option<&str> {
        match self {
            Value::Keyword(k) => Some(k),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

impl Rule {
    pub fn new() -> Self {
        Self {
            selectors: Vec::new(),
            declarations: Vec::new(),
        }
    }
}

impl Default for Rule {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn merge(&mut self, other: Stylesheet) {
        self.rules.extend(other.rules);
    }
}
