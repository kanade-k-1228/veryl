use std::collections::HashMap;

use veryl_analyzer::{Analyzer, AnalyzerError, symbol_table};
use veryl_metadata::Metadata;
use veryl_parser::Parser;
use veryl_simulator::model::Model;

#[track_caller]
fn analyze(code: &str) -> Vec<AnalyzerError> {
    symbol_table::clear();

    let metadata = Metadata::create_default("prj").unwrap();
    let parser = Parser::parse(&code, &"").unwrap();
    let analyzer = Analyzer::new(&metadata);

    let mut errors = vec![];
    errors.append(&mut analyzer.analyze_pass1(&"prj", &"", &parser.veryl));
    errors.append(&mut Analyzer::analyze_post_pass1());
    errors.append(&mut analyzer.analyze_pass2(&"prj", &"", &parser.veryl));
    let info = Analyzer::analyze_post_pass2();
    errors.append(&mut analyzer.analyze_pass3(&"prj", &"", &parser.veryl, &info));
    dbg!(&errors);
    errors
}

#[test]
fn test_comb() {
    let code = std::fs::read_to_string("tests/comb.veryl").unwrap();
    analyze(&code);

    let mut init = HashMap::new();
    init.insert("a".to_string(), 10);
    init.insert("b".to_string(), 20);
    let mut model = Model::new("Top", init);
    assert_eq!(model.get("c"), Some(30));

    model.input("a", 20);
    model.input("b", 30);
    assert_eq!(model.get("c"), Some(50));
}

#[test]
fn test_ff() {
    let code = std::fs::read_to_string("tests/ff.veryl").unwrap();
    analyze(&code);
    let mut model = Model::new("Top", HashMap::new());

    model.reset();

    assert_eq!(model.get("a"), Some(0));
    assert_eq!(model.get("b"), Some(0));

    model.clock();

    assert_eq!(model.get("a"), Some(1));
    assert_eq!(model.get("b"), Some(1));

    model.clock();

    assert_eq!(model.get("a"), Some(0));
    assert_eq!(model.get("b"), Some(2));
}
