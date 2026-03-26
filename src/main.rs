use nu_protocol::{Span, Value, engine::EngineState, record};
use nuon::{ToNuonConfig, ToStyle, to_nuon};

#[derive(Debug, PartialEq)]
struct Person {
    name: String,
    age: i64,
    scores: Vec<i64>,
}

impl From<&Person> for Value {
    fn from(p: &Person) -> Value {
        let span = Span::unknown();
        Value::record(
            record! {
                "name"   => Value::string(p.name.clone(), span),
                "age"    => Value::int(p.age, span),
                "scores" => Value::list(
                    p.scores.iter().map(|&s| Value::int(s, span)).collect(),
                    span,
                ),
            },
            span,
        )
    }
}

fn serialize(person: &Person) -> Result<String, String> {
    let value = Value::from(person);
    let config = ToNuonConfig::default()
        .style(ToStyle::Spaces(2))
        .raw_strings(true);
    let engine_state = EngineState::new();
    to_nuon(&engine_state, &value, config).map_err(|e| e.to_string())
}

fn main() {
    let person = Person {
        name: "Alice".into(),
        age: 30,
        scores: vec![95, 87, 100],
    };

    // Serialize
    let nuon_str = serialize(&person).expect("serialize failed");
    println!("NUON:\n{nuon_str}");
}
