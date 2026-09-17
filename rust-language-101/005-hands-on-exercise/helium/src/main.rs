








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










// fn even_numbers(numbers: &[i32]) -> Vec<i32> {
//     let mut array_of_even_number: Vec<i32> = Vec::new();
//     for &number in numbers {
//         if number % 2 == 0 {
//             array_of_even_number.push(number);
//         }
//     }

//     array_of_even_number
// }

// fn even_numbers_another_impl(numbers: &[i32]) -> Vec<i32> {
//     let mut array_of_even_number: Vec<i32> = Vec::new();
//     for number in numbers {
//         if number % 2 == 0 {
//             array_of_even_number.push(*number);
//         }
//     }

//     array_of_even_number
// }

// fn main() {
//     let numbers = vec![10, 15, 20, 25, 30, 35, 40];
//     let array_of_even_number: Vec<i32> = even_numbers(&numbers);
//     println!("{:?}", array_of_even_number);
// }










// fn double_numbers(numbers: &[i32]) -> Vec<i32> {
//     let mut doubled_numbers: Vec<i32> = Vec::new();
//     for &number in numbers {
//         doubled_numbers.push(2 * number);
//     }

//     doubled_numbers
// }

// fn main() {
//     let numbers: Vec<i32> = vec![1, 2, 3, 4, 5];
//     let doubled_numbers: Vec<i32> = double_numbers(&numbers);
//     println!("{:?}", numbers);
//     println!("{:?}", doubled_numbers);
// }










// fn find_number(numbers: &[i32], target: i32) -> Option<i32> {
//     for &number in numbers {
//         if number == target {
//             return Option::Some(number);
//         }
//     }

//     Option::None
// }

// fn match_the_result(result: &Option<i32>) {
//     match result {
//         Some(number) => println!("Found: {}", number),
//         None => println!("Number not found")
//     }
// }

// fn main() {
//     let numbers = vec![10, 20, 30, 40, 50];
//     let result: Option<i32> = find_number(&numbers, 30);
//     match_the_result(&result);
//     let another_result: Option<i32> = find_number(&numbers, 99);
//     match_the_result(&another_result);
// }










// fn first_even(numbers: &[i32]) -> Option<i32> {
//     for &number in numbers {
//         if number % 2 == 0 {
//             return Option::Some(number)
//         }
//     }

//     Option::None
// }

// fn main() {
//     let input_a: [i32; 6] = [11, 13, 17, 20, 25, 30];
//     println!("{:?}", first_even(&input_a));
//     let input_b: [i32; 4] = [11, 13, 17, 25];
//     println!("{:?}", first_even(&input_b));
// }









// fn first_even(numbers: &[i32]) -> Option<i32> {
//     for &number in numbers {
//         if number % 2 == 0 {
//             return Option::Some(number)
//         }
//     }

//     Option::None
// }

// fn match_the_result(result: &Option<i32>) {
//     match result {
//         Some(number) => println!("First even number: {}", number),
//         None => println!("No even not found")
//     }
// }

// fn main() {
//     let input_a: [i32; 6] = [11, 13, 17, 20, 25, 30];
//     match_the_result(&first_even(&input_a));
//     let input_b: [i32; 4] = [11, 13, 17, 25];
//     match_the_result(&first_even(&input_b));
// }




fn main() {
    println!("Hello, world!");
}
