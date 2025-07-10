use glob::glob;
use jumplist_parser::JumplistParser;

fn parse_and_print_glob(pattern: &str) {
    let paths = glob(pattern).expect("Failed to read glob pattern");
    for path in paths {
        match path {
            Ok(p) => match JumplistParser::from_path(p.to_str().unwrap()) {
                Ok(destlist) => {
                    let res = serde_json::to_string(&destlist).unwrap();
                    println!("{}", res);
                }
                Err(e) => eprintln!("Failed to parse file '{}': {}", p.display(), e),
            },
            Err(e) => eprintln!("Error reading path: {}", e),
        }
    }
}

#[cfg(test)]
#[test]
fn quick_access() {
    parse_and_print_glob("samples/other/5f7b5f1e01b83767.automaticDestinations-ms");
}

#[cfg(test)]
#[test]
fn win11_all() {
    parse_and_print_glob("samples/win11/*/*");
}

#[cfg(test)]
#[test]
fn win10_all() {
    parse_and_print_glob("samples/win10/*/*");
}

#[cfg(test)]
#[test]
fn win11_custom_destinations() {
    parse_and_print_glob("samples/win11/CustomDestinations/*");
}

#[cfg(test)]
#[test]
fn win11_automatic_destinations() {
    parse_and_print_glob("samples/win11/AutomaticDestinations/*");
}

#[cfg(test)]
#[test]
fn win10_custom_destinations() {
    parse_and_print_glob("samples/win10/CustomDestinations/*");
}

#[cfg(test)]
#[test]
fn win10_automatic_destinations() {
    parse_and_print_glob("samples/win10/AutomaticDestinations/*");
}
