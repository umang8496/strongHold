








// fn add(num_a: i32, num_b: i32) -> i32 {
//     num_a + num_b
// }

// fn multiply(num_a: i32, num_b: i32) -> i32 {
//     num_a * num_b
// }

// fn main() {
//     let a: i32 = 10;
//     let b: i32 = 20;
//     println!("Sum: {}", add(a, b));
//     println!("Product: {}", multiply(a, b));
// }










// fn greet(name: &str) -> String {
//     String::from("Hello, ") + name + "!"
// }

// fn greet_another_version(name: &str) -> String {
//     // Explicit mutation
//     let mut result = String::from("Hello, ");
//     result.push_str(name);
//     result.push('!');
//     result
// }

// fn main() {
//     let name: String = String::from("Umang");
//     let value: String = greet(&name);  // automatic conversion from &String to &str
//     println!("{}", value);
//     let another_value: String = greet_another_version(&name);
//     println!("{}", another_value);
// }










// fn length(text: &str) -> i32 {
//     text.len() as i32
// }

// fn main() {
//     let text = String::from("Hello Rust");
//     println!("{}", length(&text));
// }









// fn sum(numbers: Vec<i32>) -> i32 {
//     let mut sum: i32 = 0;
//     for number in numbers {
//         sum += number;
//     }
//     sum
// }

// fn sum_better_impl(numbers: &[i32]) -> i32 {
//     let mut sum: i32 = 0;
//     for number in numbers {
//         sum += number;
//     }
//     sum
// }


// fn main() {
//     let numbers: Vec<i32> = vec![10, 20, 30, 40, 50];
//     let better_result: i32 = sum_better_impl(&numbers);
//     println!("Better Sum: {}", better_result);
//     let result: i32 = sum(numbers);
//     println!("Sum: {}", result);
// }










// fn sum(numbers: &[i32]) -> i32 {
//     let mut sum: i32 = 0;
//     for &number in numbers {
//         sum += number;
//     }
//     sum
// }

// fn main() {
//     let numbers: Vec<i32> = vec![10, 20, 30, 40, 50];
//     let result: i32 = sum(&numbers);
//     println!("Sum: {}", result);
//     println!("Numbers: {:?}", numbers);
// }










fn main() {
    println!("Hello, world!");
}
