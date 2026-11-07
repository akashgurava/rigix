use rigix_macros::entity;
use uuid::Uuid;

#[entity]
struct UserAccount {
    id: i64,
    uuid: Uuid,
    name: String,
    active: bool,
    balance: f64,
    tags: Vec<String>,
}

fn main() {
    let dummy_uuid = Uuid::new_v4();
    let account = UserAccount {
        id: 101,
        uuid: dummy_uuid,
        name: "Alice".to_string(),
        active: true,
        balance: 250.75,
        tags: vec!["admin".to_string(), "finance".to_string()],
    };

    // 1. Primitive and Uuid copy return by value
    let id: i64 = account.id();
    assert_eq!(id, 101);

    let uuid_val: Uuid = account.uuid();
    assert_eq!(uuid_val, dummy_uuid);

    let active: bool = account.active();
    assert!(active);

    let balance: f64 = account.balance();
    assert!((balance - 250.75).abs() < f64::EPSILON);

    // 2. String returns &str
    let name: &str = account.name();
    assert_eq!(name, "Alice");

    // 3. Generic/compound types return by reference
    let tags: &Vec<String> = account.tags();
    assert_eq!(tags.len(), 2);
    assert_eq!(&tags[0], "admin");

    // 4. into_parts decomposes the entity into its fields as a tuple
    let (id_val, uuid_out, name_val, active_val, balance_val, tags_val) = account.into_parts();
    assert_eq!(id_val, 101);
    assert_eq!(uuid_out, dummy_uuid);
    assert_eq!(name_val, "Alice");
    assert!(active_val);
    assert!((balance_val - 250.75).abs() < f64::EPSILON);
    assert_eq!(tags_val, vec!["admin".to_string(), "finance".to_string()]);
}
