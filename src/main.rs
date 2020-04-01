//_____________________________________________________________________________
// Author: Garrett Tetrault
// Entry point.
//_____________________________________________________________________________
#![allow(dead_code)]

// External imports.
#[allow(unused_imports)]
use std::time::Instant;

// Internal imports.
mod operator;
mod expr;
mod population;

use operator::{Operator};
use expr::ExprParser;
use population::Population;

//_____________________________________________________________________________
//                                                                         Main
fn main() {
    //_______________________________________________________________
    //                                       Initiate and Fill Parser
    let mut parser = ExprParser::new();

    // Basic arithmetic.
    parser.insert("ADD", Operator::Binary(|x, y| x + y));
    parser.insert("SUB", Operator::Binary(|x, y| x - y));
    parser.insert("MUL", Operator::Binary(|x, y| x * y));
    parser.insert("DIV", Operator::Binary(|x, y| x / y));

    parser.insert("SQUARE", Operator::Unary(|x| x * x));
    parser.insert("SQRT", Operator::Unary(f64::sqrt));

    // Trigonometric.
    parser.insert("COS", Operator::Unary(f64::cos));
    parser.insert("SIN", Operator::Unary(f64::sin));
    parser.insert("TAN", Operator::Unary(f64::tan));

    //_______________________________________________________________
    //                                                     Given Data

    let time_data = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let position_data = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    //_______________________________________________________________
    //                                            Simulate Population

    // let expr = parser.from_description("TIME");
    // let fitness = expr.fitness(&time_data, &position_data);

    // println!("Fitness = {}", fitness);
    let mut population = Population::new(time_data, position_data);

    population.grow(&parser, 300);

    for _ in 0..10 {
        population.population.sort();
        println!("_________________________\n\
        Generation {}:", 
        population.generation);

        for individual in population.population.iter() {
            println!("{}, fitness = {}", individual.expr.description(), individual.fitness);
        }
        population.evolve();
    }

    let expr = population.best_fit();

    println!("\n**********");
    println!("Best fit = {}", expr.description());
    println!("Generation = {}", population.generation);
    println!("Population size = {}", population.population.len());
}