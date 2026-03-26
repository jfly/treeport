struct Config {
    categories: Vec<Category>,
}

struct Category {
    name: String,
    detect_command: Vec<String>,

    conditions: Vec<Condition>,
    stats: Vec<Stat>,
}

struct Condition {
    name: String,
    command: Vec<String>,
}

struct Stat {
    name: String,
    command: Vec<String>,
}
