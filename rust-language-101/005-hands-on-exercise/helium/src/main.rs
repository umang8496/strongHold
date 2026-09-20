








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









// fn double_if_present(value: Option<i32>) -> Option<i32> {
//     if value.is_some() {
//         return match value {
//             Some(number) => Option::Some(2 * number),
//             None => None
//         }
//     } else {
//         return Option::None;
//     }
// }

// fn main() {
//     let option_10: Option<i32> = Option::Some(10);
//     println!("{:?}", double_if_present(option_10));
//     let option_25: Option<i32> = Option::Some(25);
//     println!("{:?}", double_if_present(option_25));
// }










// fn find_name(names: &[String], target: &str) -> Option<String> {
//     for name in names {
//         if name == target {
//             return Option::Some(name.clone());
//         }
//     }
//     Option::None
// }

// fn main() {
//     let names = vec![
//         String::from("Alice"),
//         String::from("Bob"),
//         String::from("Charlie"),
//         String::from("David"),
//     ];

//     println!("{:?}", &find_name(&names, "Charlie"));
//     println!("{:?}", &find_name(&names, "Eve"));
// }









// struct User {
//     id: i32,
//     name: String,
//     age: i32,
// }

// impl User {
//     fn new(id: i32, name: String, age: i32) -> Self {
//         User { id, name, age }
//     }

//     fn is_adult(&self) -> bool {
//         self.age >= 18
//     }
// }

// fn main() {
//     let user = User::new(1, String::from("Umang"), 20);
//     println!("User: {}", user.name);
//     println!("Adult: {}", user.is_adult());
// }










// enum Shape {
//     Circle(f64),
//     Rectangle(f64, f64),
// }


// fn area(shape: &Shape) -> f64 {
//     match shape {
//         Shape::Circle(radius) => std::f64::consts::PI * radius * radius,
//         Shape::Rectangle(width, height) => width * height,
//     }
// }

// fn main() {
//     let circle = Shape::Circle(10.0);
//     let rectangle = Shape::Rectangle(10.0, 5.0);

//     println!("Circle area: {}", area(&circle));
//     println!("Rectangle area: {}", area(&rectangle));
// }










// struct Circle {
//     radius: f64,
// }

// struct Rectangle {
//     length: f64,
//     width: f64,
// }

// trait Area {
//     fn area(&self) -> f64;
// }

// impl Area for Circle {
//     fn area(&self) -> f64 {
//         std::f64::consts::PI * self.radius * self.radius
//     }
// }

// impl Area for Rectangle {
//     fn area(&self) -> f64 {
//         self.length * self.width
//     }
// }

// fn main() {
//     let circle = Circle { radius: 10.0 };
//     let rectangle = Rectangle {
//         length: 10.0,
//         width: 5.0,
//     };

//     println!("Circle: {}", circle.area());
//     println!("Rectangle: {}", rectangle.area());
// }









// trait Area {
//     fn area(&self) -> f64;
// }

// struct Circle {
//     radius: f64,
// }

// impl Area for Circle {
//     fn area(&self) -> f64 {
//         std::f64::consts::PI * self.radius * self.radius
//     }
// }

// struct Rectangle {
//     length: f64,
//     breadth: f64,
// }

// impl Area for Rectangle {
//     fn area(&self) -> f64 {
//         self.length * self.breadth
//     }
// }

// fn print_area<T: Area>(shape: &T) {
//     let calculated_area: f64 = shape.area();
//     println!("Area: {}", calculated_area);
// }

// fn main() {
//     let circle = Circle { radius: 10.0 };
//     let rectangle = Rectangle {
//         length: 10.0,
//         breadth: 5.0,
//     };
//     print_area(&circle);
//     print_area(&rectangle);
// }










// trait Area {
//     fn area(&self) -> f64;
// }

// struct Circle {
//     radius: f64,
// }

// impl Area for Circle {
//     fn area(&self) -> f64 {
//         std::f64::consts::PI * self.radius * self.radius
//     }
// }

// impl std::fmt::Display for Circle {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Circle {{ radius: {} }}", self.radius)
//     }
// }

// struct Rectangle {
//     length: f64,
//     breadth: f64,
// }

// impl Area for Rectangle {
//     fn area(&self) -> f64 {
//         self.length * self.breadth
//     }
// }

// impl std::fmt::Display for Rectangle {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Rectangle {{ length: {}, breadth: {} }}", self.length, self.breadth)
//     }
// }

// fn describe<T: Area + std::fmt::Display>(shape: &T) {
//     println!("Area: {}", &shape.area());
//     println!("Shape: {}", &shape);
// }

// fn main() {
//     let circle = Circle { radius: 10.0 };
//     let rectangle = Rectangle {
//         length: 10.0,
//         breadth: 5.0,
//     };
//     describe(&circle);
//     describe(&rectangle);
// }










// trait Area {
//     fn area(&self) -> f64;
// }

// #[derive(Debug)]
// struct Circle {
//     radius: f64,
// }

// impl Area for Circle {
//     fn area(&self) -> f64 {
//         std::f64::consts::PI * self.radius * self.radius
//     }
// }

// #[derive(Debug)]
// struct Rectangle {
//     length: f64,
//     breadth: f64,
// }

// impl Area for Rectangle {
//     fn area(&self) -> f64 {
//         self.length * self.breadth
//     }
// }

// fn describe(shape: &(impl Area + std::fmt::Debug)) {
//     println!("Area: {}", &shape.area());
//     println!("Shape: {:?}", &shape);
// }

// fn main() {
//     let circle = Circle { radius: 10.0 };
//     let rectangle = Rectangle {
//         length: 10.0,
//         breadth: 5.0,
//     };
//     describe(&circle);
//     describe(&rectangle);
// }









// trait Area {
//     fn area(&self) -> f64;
// }

// #[derive(Debug)]
// struct Circle {
//     radius: f64,
// }

// impl Area for Circle {
//     fn area(&self) -> f64 {
//         std::f64::consts::PI * self.radius * self.radius
//     }
// }

// #[derive(Debug)]
// struct Rectangle {
//     length: f64,
//     breadth: f64,
// }

// impl Area for Rectangle {
//     fn area(&self) -> f64 {
//         self.length * self.breadth
//     }
// }

// // This one works as well
// // fn same_area<T: Area, V: Area>(a: &T, b: &V) -> bool {
// //     a.area() == b.area()
// // }

// fn same_area(a: &impl Area, b: &impl Area) -> bool {
//     a.area() == b.area()
// }

// fn main() {
//     let circle1 = Circle { radius: 10.0 };
//     let circle2 = Circle { radius: 10.0 };
//     println!("{}", same_area(&circle1, &circle2));    // true

//     let rectangle = Rectangle {
//         length: 10.0,
//         breadth: 5.0,
//     };

//     println!("{}", same_area(&rectangle, &circle2));    // false
// }










// trait Area {
//     fn area(&self) -> f64;
// }

// #[derive(Debug)]
// struct Circle {
//     radius: f64,
// }

// impl Area for Circle {
//     fn area(&self) -> f64 {
//         std::f64::consts::PI * self.radius * self.radius
//     }
// }

// #[derive(Debug)]
// struct Rectangle {
//     length: f64,
//     breadth: f64,
// }

// impl Area for Rectangle {
//     fn area(&self) -> f64 {
//         self.length * self.breadth
//     }
// }

// fn print_areas(shapes: &Vec<&dyn Area>) {
//     for shape in shapes {
//         println!("Area: {}", shape.area());
//     } 
// }

// fn main() {
//     let circle = Circle { radius: 10.0 };
//     let rectangle = Rectangle {
//         length: 10.0,
//         breadth: 5.0,
//     };

//     let shapes: Vec<&dyn Area> = vec![
//         &circle,
//         &rectangle,
//     ];

//     print_areas(&shapes);
// }









