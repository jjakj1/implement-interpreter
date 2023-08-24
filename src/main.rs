use implement_parser::repl;
use users::{get_current_uid, get_user_by_uid};

fn main() {
    let user = get_user_by_uid(get_current_uid()).expect("Can not get current user!");
    println!(
        "Hello {:?}! This is the Monkey programming language!",
        user.name()
    );
    println!("Feel free to type in commands");
    repl::start().unwrap();
}
