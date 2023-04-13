struct Meme {
    id: u32,
    content: String,
    extra_data: String,
    summary: String,
    desc: Option<String>,
}

struct Tag {
    namespace: String,
    value: String,
}
