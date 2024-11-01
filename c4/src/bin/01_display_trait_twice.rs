use std::fmt::{Display, Formatter};

struct Person {
    name: String,
    age: u8,
}

// TODO: We want to have the ability to print the above struct both as JSON and HTML.
// What are the different ways that we can use to achieve that?
// Is there a way to do it so that we can print persons as JSON or HTML through the same trait
// (`Display`), so that we can do it through a unified interface?


fn print_something<T: Display>(t: &T) {
    println!("Printing\n{t}");
}

fn main() {
    let person = Person {
        name: "Kuba B.".to_string(),
        age: 30,
    };


    // TODO: print person as JSON through `print_something`
    // TODO: print person as HTML through `print_something`
}
