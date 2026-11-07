use rigix_macros::entity;

#[entity]
struct MultiVisEntity {
    pub standard_pub: i64,
    pub(crate) crate_pub: String,
    pub(super) super_pub: bool,
    pub(in crate) path_pub: u32,
    private_field: String,
}

fn main() {}
