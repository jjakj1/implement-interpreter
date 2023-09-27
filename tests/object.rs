use implement_parser::evaluator::object;
use implement_parser::evaluator::object::Hashable;

#[test]
fn test_string_hash_key() {
    let hello1 = Box::new(object::StringObject {
        value: "Hello World".to_owned(),
    });
    let hello2 = Box::new(object::StringObject {
        value: "Hello World".to_owned(),
    });
    let diff1 = Box::new(object::StringObject {
        value: "My name is Kafka".to_owned(),
    });
    let diff2 = Box::new(object::StringObject {
        value: "My name is Kafka".to_owned(),
    });

    assert_eq!(hello1.hash_key(), hello2.hash_key());
    assert_eq!(diff1.hash_key(), diff2.hash_key());
    assert_ne!(hello1.hash_key(), diff1.hash_key());
}
