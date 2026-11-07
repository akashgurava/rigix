use rigix_macros::entity;

#[entity]
struct ValidEntity {
    id: i64,
    name: String,
}

fn main() {
    let _ = ValidEntity {
        id: 1,
        name: "test".to_string(),
    };
}
