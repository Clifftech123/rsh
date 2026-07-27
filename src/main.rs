fn main() {
    // --- OWNERSHIP ---
    demonstrate_ownership();
    desmostrat_second_owenership();

    // --- BORROWING ---
    demonstrate_borrowing();

    // --- LIFETIMES ---
    demonstrate_lifetimes();
}

// =============================================================
// OWNERSHIP
// When a value is assigned to another variable, ownership moves.
// The original variable can no longer be used.
// =============================================================
fn demonstrate_ownership() {
    let s1 = String::from("hello"); // s1 owns this string
    let s2 = s1; // ownership moves to s2, s1 is invalid

    // println!("{s1}"); // ❌ would cause compile error - s1 no longer owns the value
    println!("Ownership - s2 now owns the value: {s2}");

    // Primitive types (i64, bool, etc.) are COPIED, not moved
    let x = 10;
    let y = x; // x is COPIED into y, both are still valid
    println!("Ownership - x: {x}, y: {y} (both valid because i64 is Copy)");
}

// Desmonstrate secnd exmapls
//
pub fn desmostrat_second_owenership() {
    let num1: String = String::from("Clifford"); // This case num one won the value right
    // Now ewe moving the owenship
    let num2: String = num1;

    //  Here can't print number one becuase is move out scope
    println!(" i have it now : {num2}")
}

// =============================================================
// BORROWING
// Borrow a value with & so you can use it without taking ownership.
// Use &mut to borrow and also modify the value.
// =============================================================
fn demonstrate_borrowing() {
    let s = String::from("world");

    let length = get_length(&s); // borrow s, s still owns the value
    println!("Borrowing - '{s}' has length {length}");

    let mut greeting = String::from("hello");
    append_word(&mut greeting); // mutable borrow - can modify the value
    println!("Borrowing - after mutation: {greeting}");
}

fn get_length(s: &String) -> usize {
    s.len() // we only borrow, we don't own or drop the value
}

fn append_word(s: &mut String) {
    s.push_str(" world"); // we can modify because we have a mutable borrow
}

// =============================================================
// LIFETIMES
// Lifetimes tell Rust how long a reference must stay valid.
// The 'a annotation means both inputs live at least as long as the output.
// =============================================================
fn demonstrate_lifetimes() {
    let s1 = String::from("long string");
    let s2 = String::from("xyz");

    let result = longest(&s1, &s2);
    println!("Lifetimes - longest string is: {result}");
}

// 'a means: the returned reference will be valid as long as BOTH s1 and s2 are valid
fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() { s1 } else { s2 }
}
