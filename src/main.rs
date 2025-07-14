use std::env;

fn getos() {
    //TODO: Get the OS information for the current system
}

fn main() {
    let path = env::current_dir();

    // TODO: After getting the OS info, handle the case where Windows needs "\" and MacOS and Linux
    // needs "/"

    println!(
        "Path to the source file: {}\\src\\main.rs",
        path.expect("NO PATH FOUND").display()
    )
}
