use std::fs;

#[test]
fn test_api_documentation_matches_routes() {
    let doc = fs::read_to_string("API_DOCUMENTATION.md")
        .expect("API_DOCUMENTATION.md should exist in project root");

    let required_routes = vec![
        ("POST", "/signup"),
        ("POST", "/login"),
        ("GET", "/characters"),
        ("POST", "/characters"),
        ("GET", "/characters/{id}/progression"),
        ("GET", "/characters/{id}/asi-history"),
        ("GET", "/characters/{id}/proficiencies"),
        ("POST", "/characters/{id}/proficiencies"),
        ("POST", "/characters/{id}/proficiencies/batch"),
        ("GET", "/characters/{id}/proficiencies/skills"),
    ];

    for (method, path) in required_routes {
        let pattern_header = format!("`{} {}`", method, path);
        let pattern_summary = format!("| `{}` | `{}` |", method, path);

        let found = doc.contains(&pattern_header)
            || doc.contains(&pattern_summary)
            || doc.contains(&format!("{} {}", method, path));

        assert!(
            found,
            "Route `{} {}` is not documented in API_DOCUMENTATION.md",
            method, path
        );
    }
}
