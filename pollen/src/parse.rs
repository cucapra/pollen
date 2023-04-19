extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "pollen.pest"]
pub struct PollenParser;

pub fn parse() {
    let successful_parse = PollenParser::parse(Rule::var, "Var1");
    println!("{:?}", successful_parse);

    let unsuccessful_parse = PollenParser::parse(Rule::var, "1v");
    println!("{:?}", unsuccessful_parse);
}