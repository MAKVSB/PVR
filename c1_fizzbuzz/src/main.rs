fn main() {
    let mut i = 0;
    loop {
        i += 1;
        if i == 100 {
            break
        }

        if (i % 15) == 0 {
            print!("FizzBuzz, ");
        } else if (i % 3) == 0 {
            print!("Fizz, ");
        } else if (i % 5) == 0 {
            print!("Buzz, ");
        } else {
            print!("{}, ", i)
        }
    }

}
