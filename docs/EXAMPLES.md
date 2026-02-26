# Examples and Patterns

Sample Axiom programs demonstrating language features and best practices.

## Basic Examples

### Hello World

**hello.ax:**
```axiom
let msg = "Hello, World!";
```

Run:
```bash
axm run hello.ax
```

### Simple Arithmetic

**math.ax:**
```axiom
let x = 10;
let y = 20;
let sum = x + y;
let product = x * y;
let quotient = y / x;
```

### Variables and Strings

**strings.ax:**
```axiom
let name = "Alice";
let age = 30;
let greeting = "Hello, " + name + "!";
```

## Control Flow

### If-Else Conditions

**conditions.ax:**
```axiom
let score = 85;

if score >= 90 {
  let grade = "A";
} else {
  let grade = "B";
}
```

### Temperature Check

**temperature.ax:**
```axiom
let temp = 25;

if temp < 0 {
  let status = "Freezing";
} else {
  if temp < 15 {
    let status = "Cold";
  } else {
    if temp < 25 {
      let status = "Cool";
    } else {
      let status = "Warm";
    }
  }
}
```

### While Loop

**loop.ax:**
```axiom
let i = 0;
while i < 5 {
  let i = i + 1;
}
```

Countdown:
```axiom
let count = 10;
while count > 0 {
  let count = count - 1;
}
```

### For Loop

**iteration.ax:**
```axiom
for item in [1, 2, 3, 4, 5] {
  // process item
}
```

Iterating strings (planned):
```axiom
for char in "hello" {
  // process each character
}
```

## Input and Output

Interactive programs that read user input and display output.

### Simple Output

**print.ax:**
```axiom
out("Hello, Axiom!");
out(42);
out(true);
out([1, 2, 3]);
```

Output:
```
Hello, Axiom!
42
true
[1, 2, 3]
```

### Reading User Input

**read_input.ax:**
```axiom
out("What is your name?");
let name = in();
out("Hello, ");
out(name);
out("!");
```

Session (note: user input is NOT echoed):
```
What is your name?
(user types "Alice" but it's not displayed)
Hello, Alice!
```

### Greeting Program

**greet.ax:**
```axiom
out("Welcome to Axiom!");
let name = in("What is your name? ");
let age = in("How old are you? ");
out("Hello, ");
out(name);
out("! You are ");
out(age);
out(" years old!");
```

Session (input is NOT echoed, only the prompt is shown):
```
Welcome to Axiom!
What is your name? (user types "Bob", not displayed)
How old are you? (user types "30", not displayed)
Hello, Bob! You are 30 years old!
```

out("Nice to meet you, ");
out(name);
out("!");

out("How old are you?");
let age_str = in();
let age = num(age_str);

if age >= 18 {
  out("You are an adult!");
} else {
  out("You are a minor!");
}
```

Session:
```
Welcome to Axiom!
What is your name?
> Bob
Nice to meet you, Bob!
How old are you?
> 25
You are an adult!
```

### Echo Program

**echo.ax:**
```axiom
out("Type something:");
let user_input = in();
out("You typed: ");
out(user_input);
```

Session:
```
Type something:
> Hello, World!
You typed: Hello, World!
```

### Multiple Inputs

**multi_input.ax:**
```axiom
out("Enter three numbers:");

out("Number 1:");
let n1 = in();

out("Number 2:");
let n2 = in();

out("Number 3:");
let n3 = in();

out("You entered: ");
out(n1);
out(", ");
out(n2);
out(", ");
out(n3);
```

Session:
```
Enter three numbers:
Number 1:
> 10
Number 2:
> 20
Number 3:
> 30
You entered: 10, 20, 30
```

### Interactive Calculator

**calculator.ax:**
```axiom
out("Simple Calculator");
out("Enter first number:");
let a = in();

out("Enter second number:");
let b = in();

out("Choose operation: +, -, *, /");
let op = in();

if op == "+" {
  out("Result: ");
  out(num(a) + num(b));
} else {
  if op == "-" {
    out("Result: ");
    out(num(a) - num(b));
  } else {
    if op == "*" {
      out("Result: ");
      out(num(a) * num(b));
    } else {
      if op == "/" {
        out("Result: ");
        out(num(a) / num(b));
      } else {
        out("Invalid operation");
      }
    }
  }
}
```

Session:
```
Simple Calculator
Enter first number:
> 10
Enter second number:
> 5
Choose operation: +, -, *, /
> +
Result: 15
```

### Printing Collections

**print_collections.ax:**
```axiom
out("List: ");
out([1, 2, 3, 4, 5]);

out("Map: ");
out({"name": "Alice", "age": 30});

out("Nested: ");
out([[1, 2], [3, 4]]);
```

Output:
```
List: [1, 2, 3, 4, 5]
Map: {name: Alice, age: 30}
Nested: [[1, 2], [3, 4]]
```

## Functions

### Basic Function

**functions.ax:**
```axiom
fun add(a, b) {
  ret a + b;
}

let result = add(10, 20);  // 30
```

### Function with Conditions

**fibonacci.ax:**
```axiom
fun fib(n) {
  if n <= 1 {
    ret n;
  } else {
    ret fib(n - 1) + fib(n - 2);
  }
}

let f5 = fib(5);  // 5
let f10 = fib(10); // 55
```

### Factorial

**factorial.ax:**
```axiom
fun factorial(n) {
  if n <= 1 {
    ret 1;
  } else {
    ret n * factorial(n - 1);
  }
}

let fact5 = factorial(5);   // 120
let fact10 = factorial(10); // 3628800
```

### Nested Functions

**nested.ax:**
```axiom
fun outer(x) {
  fun inner(y) {
    ret x + y;
  }
  ret inner(5);
}

let result = outer(10);  // 15
```

### Function Returning Boolean

**is_even.ax:**
```axiom
fun is_even(n) {
  ret n % 2 == 0;
}

if is_even(10) {
  let msg = "Ten is even";
}
```

## Lists and Collections

### List Creation

**lists.ax:**
```axiom
let empty = [];
let numbers = [1, 2, 3, 4, 5];
let strings = ["apple", "banana", "cherry"];
let nested = [[1, 2], [3, 4], [5, 6]];
```

### List Indexing

**indexing.ax:**
```axiom
let items = [10, 20, 30, 40, 50];
let first = items[0];   // 10
let third = items[2];   // 30
let last = items[4];    // 50
```

### List Iteration

**list_iteration.ax:**
```axiom
let items = [2, 4, 6, 8, 10];
for num in items {
  // process each number
}
```

### Nested Lists

**nested_lists.ax:**
```axiom
let matrix = [
  [1, 2, 3],
  [4, 5, 6],
  [7, 8, 9]
];

let row0_col1 = matrix[0][1];  // 2
let row2_col2 = matrix[2][2];  // 9
```

### List Bounds

**bounds_checking.ax:**
```axiom
let nums = [1, 2, 3];
let valid = nums[0];   // 1, OK
let invalid = nums[5]; // error: index out of bounds
```

## Algorithms

### Summation

**sum.ax:**
```axiom
fun sum_list(nums) {
  let total = 0;
  for item in nums {
    let total = total + item;
  }
  ret total;
}

let result = sum_list([1, 2, 3, 4, 5]);  // 15
```

### Searching

**search.ax:**
```axiom
fun find_index(list, target) {
  let i = 0;
  while i < len(list) {
    if list[i] == target {
      ret i;
    }
    let i = i + 1;
  }
  ret -1;  // not found
}

let idx = find_index([10, 20, 30, 40], 30);  // 2
```

### Counting

**count.ax:**
```axiom
fun count_even(numbers) {
  let count = 0;
  for num in numbers {
    if num % 2 == 0 {
      let count = count + 1;
    }
  }
  ret count;
}

let even_count = count_even([1, 2, 3, 4, 5, 6]);  // 3
```

### Linear Search

**linear_search.ax:**
```axiom
fun contains(list, value) {
  for item in list {
    if item == value {
      ret true;
    }
  }
  ret false;
}

if contains([1, 5, 10, 15], 10) {
  let msg = "Found!";
}
```

## Mathematical Examples

### Prime Checker

**prime.ax:**
```axiom
fun is_prime(n) {
  if n < 2 {
    ret false;
  }
  let i = 2;
  while i * i <= n {
    if n % i == 0 {
      ret false;
    }
    let i = i + 1;
  }
  ret true;
}

if is_prime(17) {
  let msg = "17 is prime";
}
```

### GCD (Greatest Common Divisor)

**gcd.ax:**
```axiom
fun gcd(a, b) {
  while b != 0 {
    let temp = b;
    let b = a % b;
    let a = temp;
  }
  ret a;
}

let result = gcd(48, 18);  // 6
```

### Power Function

**power.ax:**
```axiom
fun power(base, exp) {
  if exp == 0 {
    ret 1;
  }
  if exp == 1 {
    ret base;
  }
  ret base * power(base, exp - 1);
}

let result = power(2, 8);  // 256
```

## Practical Examples

### Temperature Converter

**temperature_converter.ax:**
```axiom
fun celsius_to_fahrenheit(c) {
  ret c * 9 / 5 + 32;
}

fun fahrenheit_to_celsius(f) {
  ret (f - 32) * 5 / 9;
}

let temp_f = celsius_to_fahrenheit(25);  // 77
let temp_c = fahrenheit_to_celsius(77);  // 25
```

### Distance Calculator

**distance.ax:**
```axiom
fun distance(x1, y1, x2, y2) {
  let dx = x2 - x1;
  let dy = y2 - y1;
  let sum = dx * dx + dy * dy;
  ret sqrt(sum);
}

let d = distance(0, 0, 3, 4);  // 5
```

### Grade Calculator

**grade_calculator.ax:**
```axiom
fun calculate_grade(score) {
  if score >= 90 {
    ret "A";
  } else {
    if score >= 80 {
      ret "B";
    } else {
      if score >= 70 {
        ret "C";
      } else {
        if score >= 60 {
          ret "D";
        } else {
          ret "F";
        }
      }
    }
  }
}

let grade = calculate_grade(85);  // "B"
```

### Palindrome Checker

**palindrome.ax:**
```axiom
fun is_palindrome(text) {
  let left = 0;
  let right = len(text) - 1;
  
  while left < right {
    if text[left] != text[right] {
      ret false;
    }
    let left = left + 1;
    let right = right - 1;
  }
  ret true;
}

if is_palindrome("racecar") {
  let msg = "It's a palindrome!";
}
```

## Advanced Patterns

### Scope Shadowing

**shadowing.ax:**
```axiom
let x = 10;
{
  let x = 20;  // shadows outer x
  // x is 20 in this block
}
// x is still 10 here
```

### Closures (Planned)

**closures.ax:**
```axiom
fun make_multiplier(factor) {
  fun multiply(x) {
    ret x * factor;
  }
  ret multiply;
}

let double = make_multiplier(2);
let result = double(5);  // 10
```

### Higher-Order Functions (Planned)

**map_example.ax:**
```axiom
fun map(list, func) {
  let result = [];
  for item in list {
    let result = result + [func(item)];
  }
  ret result;
}

fun double(x) { ret x * 2; }
let doubled = map([1, 2, 3], double);  // [2, 4, 6]
```

## Performance Considerations

### Efficient List Processing

```axiom
// Good: iterate once
fun process_items(items) {
  let result = [];
  for item in items {
    let result = result + [transform(item)];
  }
  ret result;
}

// Avoid: repeated iterations
fun bad_process(items) {
  let temp = [];
  for item in items {
    let temp = temp + [item];
  }
  for item in temp {
    let result = result + [transform(item)];
  }
  ret result;
}
```

### Tail Recursion (Planned)

```axiom
fun sum_helper(nums, idx, acc) {
  if idx >= len(nums) {
    ret acc;
  }
  ret sum_helper(nums, idx + 1, acc + nums[idx]);
}

fun sum_efficient(nums) {
  ret sum_helper(nums, 0, 0);
}
```

## Testing Patterns

### Assertion Checks

**test_math.ax:**
```axiom
fun test_add() {
  let result = add(2, 3);
  if result != 5 {
    // test failed
  }
}

fun test_multiply() {
  let result = multiply(4, 5);
  if result != 20 {
    // test failed
  }
}
```

## Common Idioms

### Guard Clause

```axiom
fun process(value) {
  if value == nil {
    ret nil;
  }
  // process value
  ret result;
}
```

### Early Return

```axiom
fun validate(input) {
  if is_empty(input) {
    ret false;
  }
  if is_invalid(input) {
    ret false;
  }
  ret true;
}
```

### Accumulator Pattern

```axiom
fun sum_numbers(nums) {
  let acc = 0;
  for num in nums {
    let acc = acc + num;
  }
  ret acc;
}
```

## See Also

- [Language Reference](LANGUAGE.md) - Full syntax
- [Type System](TYPES.md) - Type rules
- [Standard Library](STDLIB.md) - Built-in functions
- [Getting Started](GETTING_STARTED.md) - Tutorial
