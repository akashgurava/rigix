use rigix_macros::entity;

#[entity]
struct InvalidEntity {
    pub id: i64,
    name: String,
}

fn main() {}
