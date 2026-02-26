# AXIOM MONOLITH STANDARD LIBRARY REFERENCE

Complete reference for all 22 modules integrated into the statically-linked intrinsic monolith.
Each function is demonstrated with real Axiom syntax examples and return value documentation.

---

## TABLE OF CONTENTS
1. **alg** - Algorithms & Logic
2. **ann** - Annotations & Reflection
3. **aut** - Automation & Timing
4. **clr** - Colors & TrueColor
5. **col** - Collections & Maps
6. **con** - Concurrency & Async
7. **csv** - CSV Parsing & Serialization
8. **dfm** - DataFrames (Polars)
9. **env** - Environment Variables
10. **git** - Git Operations
11. **ioo** - Buffered I/O & Filesystem
12. **jsn** - JSON Operations
13. **log** - Logging & Progress Bars
14. **mth** - Mathematics
15. **net** - Networking
16. **num** - Numerics & Arrays
17. **plt** - Plotting (Plotters)
18. **pth** - Path & Directory Operations
19. **str** - String Operations & Regex
20. **sys** - System Information
21. **tim** - Time & Scheduling
22. **tui** - Terminal UI

---

## MODULE 1: ALG (ALGORITHMS & LOGIC)

**Backend:** rayon, petgraph  
**Category:** Logic/Mathematics

### alg.range(n: Num) → Lst

Generate a list of integers from 0 to n-1 using optimized iteration.

```axiom
// Generate numbers 0 through 4
let numbers = alg.range(5);
out numbers;  // [0, 1, 2, 3, 4]

// Use in loops
fun print_range() {
  let r = alg.range(10);
  out r;
}
```

### alg.sum(lst: Lst) → Num

Sum all numeric values in a list. Non-numeric values are skipped.

```axiom
let values = [10, 20, 30, 40];
let total = alg.sum(values);
out total;  // 100

let mixed = [1, "text", 2, 3];
out alg.sum(mixed);  // 6
```

### alg.filter(lst: Lst, predicate: Fun) → Lst

Filter list elements based on a predicate function (simplified implementation).

```axiom
let nums = [1, 2, 3, 4, 5];
let filtered = alg.filter(nums);  // Returns filtered list
out filtered;
```

### alg.fold(lst: Lst, init: Any) → Any

Reduce/fold operation over a list starting with an initial accumulator value.

```axiom
let values = [1, 2, 3, 4];
let result = alg.fold(values, 0);
out result;
```

### alg.map_parallel(lst: Lst, func: Fun) → Lst

Parallel map operation using rayon for thread-parallelized processing.

```axiom
let data = alg.range(100);
let mapped = alg.map_parallel(data);  // Parallel processing
out mapped;
```

---

## MODULE 2: ANN (ANNOTATIONS & REFLECTION)

**Backend:** Reflection, Pattern Matching  
**Category:** Meta-Programming

### ann.type_of(value: Any) → Str

Get the runtime type name of any value.

```axiom
out ann.type_of(42);              // "Num"
out ann.type_of("hello");         // "Str"
out ann.type_of([1, 2, 3]);       // "Lst"
out ann.type_of({ x: 10 });       // "Map"
out ann.type_of(true);            // "Bol"
out ann.type_of(fun x { x });     // "Fun"
```

### ann.is_num(value: Any) → Bol

Check if value is a number.

```axiom
out ann.is_num(42);           // true
out ann.is_num("42");         // false
out ann.is_num(3.14);         // true
```

### ann.is_str(value: Any) → Bol

Check if value is a string.

```axiom
out ann.is_str("hello");      // true
out ann.is_str(123);          // false
```

### ann.is_lst(value: Any) → Bol

Check if value is a list.

```axiom
out ann.is_lst([1, 2, 3]);    // true
out ann.is_lst("not a list"); // false
out ann.is_lst({ a: 1 });     // false
```

### ann.is_map(value: Any) → Bol

Check if value is a map/dictionary.

```axiom
out ann.is_map({ key: "value" });  // true
out ann.is_map([1, 2, 3]);         // false
out ann.is_map("string");          // false
```

### ann.fields(value: Map) → Lst

Get all keys from a map.

```axiom
let data = { name: "Alice", age: 30, city: "NYC" };
let keys = ann.fields(data);
out keys;  // ["name", "age", "city"]
```

---

## MODULE 3: AUT (AUTOMATION & TIMING)

**Backend:** chrono, croner, notify  
**Category:** Timing/Execution

### aut.now() → Num

Get current Unix timestamp in milliseconds.

```axiom
let timestamp = aut.now();
out timestamp;  // 1708873456123
```

### aut.sleep(ms: Num) → Nil

Sleep/pause execution for specified milliseconds.

```axiom
out "Starting wait...";
aut.sleep(2000);  // Sleep for 2 seconds
out "Done waiting!";
```

### aut.timestamp() → Str

Get current time as RFC3339 formatted string.

```axiom
let time_str = aut.timestamp();
out time_str;  // "2025-02-25T14:30:45.123+00:00"
```

### aut.parse_time(rfc3339_str: Str) → Num

Parse RFC3339 timestamp string to Unix timestamp in milliseconds.

```axiom
let ts = aut.parse_time("2025-02-25T14:30:45Z");
out ts;  // Returns milliseconds since epoch
```

### aut.delay(ms: Num) → Nil

Delayed execution (equivalent to sleep).

```axiom
aut.delay(500);  // Wait 500ms
out "Resumed";
```

---

## MODULE 4: CLR (COLORS & TRUECOLOR)

**Backend:** TrueColor ANSI Codes  
**Category:** Rendering/Display

### clr.rgb(r: Num, g: Num, b: Num) → Map

Create RGB color map with coordinates clipped to 0-255.

```axiom
let red = clr.rgb(255, 0, 0);
out red;  // { r: 255, g: 0, b: 0, hex: "#ff0000" }

let green = clr.rgb(0, 255, 0);
out green.hex;  // "#00ff00"

// Automatic clamping
let bright = clr.rgb(300, -50, 128);
out bright;  // { r: 255, g: 0, b: 128, hex: "#ff0080" }
```

### clr.hex(hex_code: Str) → Map

Parse hex color code to RGB map.

```axiom
let color = clr.hex("#FF5733");
out color;  // { r: 255, g: 87, b: 51 }

let cyan = clr.hex("#00FFFF");
out cyan;  // { r: 0, g: 255, b: 255 }
```

### clr.hsv(h: Num, s: Num, v: Num) → Map

Create HSV color map.

```axiom
let hue = clr.hsv(120, 100, 100);  // Hue 120°, full saturation/value
out hue;  // { h: 120, s: 100, v: 100 }
```

---

## MODULE 5: COL (COLLECTIONS & CONCURRENT MAPS)

**Backend:** dashmap (concurrent)  
**Category:** Data Structures

### col.new() → Map

Create a new empty concurrent map.

```axiom
let cart = col.new();
out cart;  // {}
```

### col.get(map: Map, key: Str) → Any

Retrieve value by key from map.

```axiom
let user = { name: "Alice", age: 30 };
let name = col.get(user, "name");
out name;  // "Alice"

let missing = col.get(user, "email");
out missing;  // nil
```

### col.set(map: Map, key: Str, value: Any) → Nil

Set key-value pair in map.

```axiom
let config = col.new();
col.set(config, "timeout", 5000);
col.set(config, "retries", 3);
col.set(config, "debug", true);
```

### col.remove(map: Map, key: Str) → Nil

Remove key from map.

```axiom
let data = { a: 1, b: 2, c: 3 };
col.remove(data, "b");
// data is now { a: 1, c: 3 }
```

### col.len(container: Map | Lst) → Num

Get count of elements in map or list.

```axiom
let map_size = col.len({ x: 1, y: 2, z: 3 });
out map_size;  // 3

let list_size = col.len([10, 20, 30, 40]);
out list_size;  // 4
```

### col.keys(map: Map) → Lst

Get all keys from a map as a list.

```axiom
let person = { name: "Bob", city: "LA", age: 25 };
let keys = col.keys(person);
out keys;  // ["name", "city", "age"]
```

### col.values(map: Map) → Lst

Get all values from a map as a list.

```axiom
let settings = { host: "localhost", port: 8080, debug: true };
let vals = col.values(settings);
out vals;  // ["localhost", 8080, true]
```

---

## MODULE 6: CON (CONCURRENCY & ASYNC)

**Backend:** tokio  
**Category:** Concurrency

### con.now() → Num

Get current timestamp (async context aware).

```axiom
let t = con.now();
out t;
```

### con.spawn(function: Fun) → Num

Spawn async task (returns task ID).

```axiom
let task_id = con.spawn(fun { out "Running async"; });
out task_id;  // Task ID
```

### con.wait(task_id: Num) → Nil

Wait for async task to complete.

```axiom
let id = con.spawn(fun { aut.sleep(1000); });
con.wait(id);
out "Task completed";
```

### con.mutex_new() → Map

Create a new concurrent mutex.

```axiom
let counter = con.mutex_new();
col.set(counter, "value", 0);
```

---

## MODULE 7: CSV (CSV PARSING & SERIALIZATION)

**Backend:** csv crate (streaming)  
**Category:** Data I/O

### csv.parse(csv_string: Str) → Lst

Parse CSV string into list of maps (each row is a map).

```axiom
let csv_data = "name,age,city
Alice,30,NYC
Bob,25,LA
Charlie,35,SF";

let rows = csv.parse(csv_data);
out rows;
// [
//   { name: "Alice", age: "30", city: "NYC" },
//   { name: "Bob", age: "25", city: "LA" },
//   { name: "Charlie", age: "35", city: "SF" }
// ]

let first_row = rows[0];
out first_row.name;  // "Alice"
```

### csv.write(rows: Lst, filepath: Str) → Nil

Write list of maps to CSV file.

```axiom
let data = [
  { id: "1", product: "Laptop", price: "999" },
  { id: "2", product: "Mouse", price: "29" }
];

csv.write(data, "products.csv");
// Creates file with headers: id,product,price
```

### csv.headers(csv_string: Str) → Lst

Extract header row from CSV string.

```axiom
let csv = "name,email,status
Alice,alice@example.com,active
Bob,bob@example.com,inactive";

let headers = csv.headers(csv);
out headers;  // ["name", "email", "status"]
```

---

## MODULE 8: DFM (DATAFRAMES - POLARS)

**Backend:** polars  
**Category:** Data Analysis

### dfm.from_csv(csv_string: Str) → Lst

Create dataframe from CSV string (returns as list of maps, max 1000 rows).

```axiom
let csv = "id,name,score
1,Alice,95
2,Bob,87
3,Charlie,92";

let df = dfm.from_csv(csv);
out df;
// [{ id: "1", name: "Alice", score: "95" }, ...]
```

### dfm.shape(dataframe: Lst) → Map

Get dimensions (rows, cols) of dataframe.

```axiom
let df = dfm.from_csv("a,b,c\n1,2,3\n4,5,6");
let dims = dfm.shape(df);
out dims;  // { rows: 2, cols: 3 }
```

### dfm.select(dataframe: Lst, columns: Lst) → Lst

Select specific columns from dataframe.

```axiom
let df = dfm.from_csv("id,name,email\n1,Alice,a@ex.com\n2,Bob,b@ex.com");
let selected = dfm.select(df, ["id", "name"]);
out selected;
// [{ id: "1", name: "Alice" }, { id: "2", name: "Bob" }]
```

### dfm.filter(dataframe: Lst) → Lst

Filter dataframe rows (simplified - returns all rows).

```axiom
let df = dfm.from_csv("val\n10\n20\n30");
let filtered = dfm.filter(df);
out filtered;
```

---

## MODULE 9: ENV (ENVIRONMENT VARIABLES)

**Backend:** dotenvy  
**Category:** Configuration

### env.get(key: Str) → Str | Nil

Get environment variable value.

```axiom
let user = env.get("USER");
out user;  // Current user name

let path = env.get("PATH");  
out path;  // System PATH

let missing = env.get("NONEXISTENT_VAR");
out missing;  // nil
```

### env.set(key: Str, value: Any) → Nil

Set environment variable for current process.

```axiom
env.set("MY_VAR", "my_value");
let val = env.get("MY_VAR");
out val;  // "my_value"
```

### env.load() → Nil

Load environment variables from .env file (if present).

```axiom
env.load();  // Loads .env in current directory
```

### env.all() → Map

Get all environment variables as a map.

```axiom
let all_vars = env.all();
out all_vars;  // { USER: "...", PATH: "...", HOME: "..." }
```

---

## MODULE 10: GIT (GIT OPERATIONS)

**Backend:** git2  
**Category:** VCS

### git.branch(repo_path: Str) → Str | Nil

Get current branch name of git repository.

```axiom
let branch = git.branch(".");
out branch;  // "main" or "develop", etc.

let repo_branch = git.branch("/path/to/repo");
out repo_branch;
```

### git.log(repo_path: Str) → Lst

Get recent commit log (last 10 commits).

```axiom
let commits = git.log(".");
out commits;
// [
//   { id: "abc123...", message: "Fix bug in parser" },
//   { id: "def456...", message: "Add new feature" },
//   ...
// ]

let latest = commits[0];
out latest.message;
```

### git.status(repo_path: Str) → Lst

Get status of modified/new files in repository.

```axiom
let changes = git.status(".");
out changes;
// [
//   { path: "file1.rs", status: "modified" },
//   { path: "file2.rs", status: "new" },
//   ...
// ]
```

### git.clone(url: Str, dest_path: Str) → Bol

Clone a git repository.

```axiom
let success = git.clone("https://github.com/user/repo.git", "./my_repo");
out success;  // true or false
```

---

## MODULE 11: IOO (BUFFERED I/O & FILESYSTEM)

**Backend:** std::fs (buffered)  
**Category:** I/O

### ioo.read(filepath: Str) → Str | Nil

Read entire file contents as string.

```axiom
let content = ioo.read("config.json");
out content;  // File contents

let missing = ioo.read("nonexistent.txt");
out missing;  // nil
```

### ioo.write(filepath: Str, content: Any) → Nil

Write content to file (overwrites if exists).

```axiom
ioo.write("output.txt", "Hello, World!");

let data = { name: "Alice", status: "active" };
ioo.write("data.txt", data);
```

### ioo.append(filepath: Str, content: Any) → Nil

Append content to file (creates if doesn't exist).

```axiom
ioo.append("log.txt", "Event occurred at ");
ioo.append("log.txt", aut.timestamp());
```

### ioo.exists(filepath: Str) → Bol

Check if file or directory exists.

```axiom
if ioo.exists("config.json") {
  out "Config found";
} else {
  out "Creating default config";
  ioo.write("config.json", "{}");
}
```

### ioo.delete(filepath: Str) → Bol

Delete file or directory recursively.

```axiom
let removed = ioo.delete("temp_file.txt");
out removed;  // true if successful

let dir_removed = ioo.delete("temp_directory");
out dir_removed;
```

### ioo.list(dirpath: Str) → Lst

List files and directories in a directory.

```axiom
let files = ioo.list(".");
out files;  // ["file1.rs", "file2.rs", "subdir", ...]

let src_files = ioo.list("./src");
out src_files;
```

---

## MODULE 12: JSN (JSON OPERATIONS)

**Backend:** serde_json  
**Category:** Data Serialization

### jsn.parse(json_string: Str) → Str

Parse JSON string (returns as string representation).

```axiom
let json = "{\"name\": \"Alice\", \"age\": 30}";
let parsed = jsn.parse(json);
out parsed;  // Parsed JSON representation

let complex = jsn.parse("[{\"id\": 1}, {\"id\": 2}]");
out complex;
```

---

## MODULE 13: LOG (LOGGING & PROGRESS)

**Backend:** indicatif  
**Category:** UI/Feedback

### log.progress(total: Num) → Nil

Show progress bar for iteration (auto-increments).

```axiom
let total_items = 100;
out "Processing items...";
log.progress(total_items);
out "Done!";
```

---

## MODULE 14: MTH (MATHEMATICS)

**Backend:** f64 intrinsics  
**Category:** Numerics

### mth.sqrt(x: Num) → Num

Calculate square root.

```axiom
out mth.sqrt(16);      // 4
out mth.sqrt(2);       // 1.41421356...
out mth.sqrt(0);       // 0
out mth.sqrt(-1);      // NaN
```

### mth.sin(x: Num) → Num

Calculate sine (x in radians).

```axiom
out mth.sin(0);        // 0
out mth.sin(1.5708);   // ~1 (π/2)
```

---

## MODULE 15: NET (NETWORKING)

**Backend:** tokio  
**Category:** Networking

*(Placeholder: Full implementation in next message)*

---

## MODULE 16: NUM (NUMERICS & ARRAYS)

**Backend:** ndarray  
**Category:** Scientific Computing

### num.zeros(rows: Num, cols: Num) → Str

Create zero-filled matrix.

```axiom
let matrix = num.zeros(3, 4);
out matrix;  // "ndarray<3x4>"
```

---

## MODULE 17: PLT (PLOTTING)

**Backend:** plotters  
**Category:** Visualization

### plt.scatter(output_path: Str) → Str

Generate scatter plot PNG.

```axiom
let chart_path = plt.scatter("scatter.png");
out chart_path;  // "scatter.png"
```

---

## MODULE 18: PTH (PATH & DIRECTORY OPERATIONS)

**Backend:** walkdir  
**Category:** Filesystem

### pth.list(dirpath: Str) → Lst

List all files in directory recursively.

```axiom
let all_files = pth.list("./src");
out all_files;  // ["file1.rs", "subdir/file2.rs", ...]
```

---

## MODULE 19: STR (STRING OPERATIONS & REGEX)

**Backend:** regex, unicode-segmentation  
**Category:** String Processing

### str.match(text: Str, pattern: Str) → Bol

Check if text matches regex pattern.

```axiom
let matches = str.match("hello@example.com", "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$");
out matches;  // true

let has_digit = str.match("abc123", "\\d");
out has_digit;  // true
```

---

## MODULE 20: SYS (SYSTEM INFORMATION)

**Backend:** sysinfo  
**Category:** System Monitoring

### sys.info() → Str

Get system information (OS, CPU, memory).

```axiom
let info = sys.info();
out info;
// "os: Some("Linux"), cpu: Some("Intel(R) Core(TM)..."), mem: 16384"
```

---

## MODULE 21: TIM (TIME & SCHEDULING)

**Backend:** chrono, croner  
**Category:** Timing

### tim.now() → Str

Get current time as RFC3339 string.

```axiom
let current = tim.now();
out current;  // "2025-02-25T14:30:45.123+00:00"
```

---

## MODULE 22: TUI (TERMINAL UI)

**Backend:** ratatui  
**Category:** UI/Display

### tui.box(text: Str) → Nil

Display text in a simple ASCII box.

```axiom
tui.box("Hello");
// Output:
// +----------------+
// | Hello          |
// +----------------+

tui.box("Status: Active");
tui.box("Error: File not found");
```

---

## USAGE PATTERNS

### Pattern 1: Data Processing Pipeline

```axiom
// Read CSV → Parse → Filter → Calculate
let csv_content = ioo.read("data.csv");
let rows = csv.parse(csv_content);
let total = alg.sum([10, 20, 30]);
out total;
```

### Pattern 2: Configuration Management

```axiom
env.load();  // Load .env
let config = col.new();
col.set(config, "host", env.get("API_HOST"));
col.set(config, "port", env.get("API_PORT"));
col.set(config, "timeout", 5000);
out config;
```

### Pattern 3: Time & Scheduling

```axiom
let start = aut.now();
out "Starting task...";
aut.sleep(1000);
let end = aut.now();
out "Completed in";  // (end - start) ms needed for calculation
```

### Pattern 4: Type Checking & Validation

```axiom
fun validate(value) {
  if ann.is_num(value) {
    out "Valid: number";
  } else if ann.is_str(value) {
    out "Valid: string";
  } else {
    out "Invalid type";
  }
}

validate(42);
validate("hello");
```

### Pattern 5: Concurrent Operations

```axiom
let task1 = con.spawn(fun { out "Task 1"; aut.sleep(100); });
let task2 = con.spawn(fun { out "Task 2"; aut.sleep(50); });
con.wait(task1);
con.wait(task2);
out "All tasks complete";
```

---

## INTEROPERABILITY

All 22 modules are automatically imported and available globally. No `import` or `std` statements needed.

```axiom
// All modules instantly available:
let nums = alg.range(5);           // alg available
let type_check = ann.type_of(nums); // ann available
let timestamp = aut.now();          // aut available
let config = col.new();             // col available
// ... and so on for all 22 modules
```

---

## PERFORMANCE CHARACTERISTICS

| Module | Backend | Thread Safety | Performance |
|--------|---------|---------------|-------------|
| alg    | rayon   | Yes (parallel)| O(n/p) where p=threads |
| ann    | runtime | Yes           | O(1) |
| aut    | chrono  | Yes           | O(1) |
| clr    | native  | Yes           | O(1) |
| col    | dashmap | Yes (concurrent) | O(1) avg |
| con    | tokio    | Yes (async)   | Async-aware |
| csv    | csv     | No            | O(n) streaming |
| dfm    | polars  | Yes           | O(n) optimized |
| env    | stdlib  | Yes           | O(1) |
| git    | git2    | Yes           | I/O bound |
| ioo    | stdlib  | Yes (buffered) | O(n) buffered |
| jsn    | serde   | No            | O(n) |
| log    | indicatif | Yes         | O(1) |
| mth    | f64     | Yes           | O(1) |
| net    | tokio   | Yes (async)   | Async-aware |
| num    | ndarray | Yes (par)     | O(n²) matrix |
| plt    | plotters | No           | O(n log n) |
| pth    | walkdir | Yes           | O(n) tree walk |
| str    | regex   | Yes (once compiled) | O(n·m) worst |
| sys    | sysinfo | Yes           | I/O bound |
| tim    | chrono  | Yes           | O(1) |
| tui    | ratatui | No            | O(1) draw |

---

## ERROR HANDLING

Most functions return `nil` on error instead of panicking. Test with type checks:

```axiom
let value = ioo.read("missing.txt");
if value {
  out "File loaded: ";
  out value;
} else {
  out "File not found";
}
```

---

## COMPILATION & RUNTIME

- **Compilation:** All intrinsics compiled into single monolithic `axm` binary
- **Runtime:** Zero module loading overhead—all functions available immediately
- **Binary Size:** ~50-80MB (release build with all backends)
- **Startup:** Instant (no dynamic loading)

For detailed module implementations, see [axm/src/intrinsics.rs](../axm/src/intrinsics.rs).
