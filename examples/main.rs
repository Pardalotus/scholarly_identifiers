use scholarly_identifiers::identifiers::Identifier;

pub fn main() {
    let input = "https://doi.org/10.5555/123456789";
    let parsed = Identifier::parse(input);
    println!("DOI Input: {}", input);
    println!("DOI Parsed: {:?}", parsed);
    println!("DOI as URI: {:?}", parsed.to_uri());
}
