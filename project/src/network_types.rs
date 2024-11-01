use enigo::{Button, Coordinate, Key};
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum Command {
    KeyPress { key: Key },
    KeyRelease { key: Key },

    MouseClick { button: Button },
    MouseMove { x: i32, y: i32, coord: Coordinate },
    MouseWheel { x: i32, y: i32 },
    MousePress { button: Button },
    MouseRelease { button: Button },
}
