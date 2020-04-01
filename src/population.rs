//_____________________________________________________________________________
// Author: Garrett Tetrault
// Test ground (for now) of expresion parsing for a genetic program.
//_____________________________________________________________________________
//external imports.
use std::cmp::Ordering;

use rand::Rng;
use rand_distr::Exp;

// Internal imports.
use crate::expr::{
    Expr,
    ExprParser,
};

//_____________________________________________________________________________
//                                                       Individual Type & Impl

#[derive(Clone)]
pub struct Individual<'a> {
    pub fitness: f64,
    pub expr: Expr<'a>,
}

// Implement an ordering to allow for sorting.

impl<'a> Ord for Individual<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        return match (self.fitness.is_nan(), other.fitness.is_nan()) {
            (true, true) => Ordering::Equal,
            (_, true) => Ordering::Less,
            (true, _) => Ordering::Greater,
            (_, _) => self.fitness.partial_cmp(&other.fitness).unwrap(),
        };
    }
}

impl<'a> PartialOrd for Individual<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}

impl<'a> PartialEq for Individual<'a> {
    fn eq(&self, other: &Self) -> bool {
        return self.fitness == other.fitness;
    }
}

impl<'a> Eq for Individual<'a> { }


//_____________________________________________________________________________
//                                                       Population Type & Impl

pub struct Population<'a> {
    // Data we are trying to fit.
    time_data: Vec<f64>,
    position_data: Vec<f64>,

    // Information on the population.
    pub population: Vec<Individual<'a>>,
    pub generation: u64,
}

impl<'a> Population<'a> {

    /* new
    */
    pub fn new(time_data: Vec<f64>, position_data: Vec<f64>) -> Population<'a> {
        if time_data.len() != position_data.len() {
            panic!(
                "Time and position data \
                must be of equal lengths.");
        }
        if time_data.len() == 0 {
            panic!(
                "Time and position data \
                cannot be emtpy.");
        }

        let population = Vec::new();
        let generation = 0;
        return Population {
            time_data, position_data, 
            population, generation
        };
    }

    /* grow
    * Grow the population by the specified number of individuals.
    */
    pub fn grow(&mut self, parser: &'a ExprParser<'a>, n: usize) {
        for _ in 0..n {
            let expr = parser.generate();
            let fitness = expr.fitness(&self.time_data, &self.position_data);
            
            let individual = Individual {fitness, expr: expr};
            self.population.push(individual);
        }
    }

    /* best_fit
    */
    pub fn best_fit(&mut self) -> Expr<'a> {
        self.population.sort();
        let individual = self.population.iter().next().unwrap();
        return individual.expr.clone();
    }

    /* evolve
    */
    pub fn evolve(&mut self) {
        self.population.sort();
        let pop_len = self.population.len();
        let min_fitness = self.population.iter().min().unwrap().fitness;

        // Build new population and keep the top 10% unchagned.
        let num_unchanged = self.population.len() / 10;
        let mut new_population = self.population[0..num_unchanged].to_vec();

        // Initialize random number generator.
        let mut rng = rand::thread_rng();

        let lambda = 0.1;
        let exp_dist = Exp::new(lambda).unwrap();

        // Generate the rest of the new population by crossover.
        for _ in 0..(pop_len - num_unchanged) {
            // Generate random fitnesses.
            let rand1 = rng.sample(exp_dist) + min_fitness;
            let rand2 = rng.sample(exp_dist) + min_fitness;

            let ind1 = self.closest(rand1);
            let ind2 = self.closest(rand2);

            let base_expr = &ind1.expr;
            let sub_expr = ind2.expr.sub_expr();

            let expr = base_expr.crossover(sub_expr);
            let fitness = expr.fitness(&self.time_data, &self.position_data);

            let individual = Individual {fitness, expr: expr};
            new_population.push(individual);
        }

        self.population = new_population;
        self.generation += 1;
    }

    /* closest
    * Find the individual with a fitness closest to the given value.
    */
    fn closest(&self, num: f64) -> &Individual<'a> {
        let mut pop_iter = self.population.iter();

        let mut prev_ind = pop_iter.next();
        let mut next_ind = pop_iter.next();

        while next_ind != None {
            if prev_ind.unwrap().fitness <= num &&
                next_ind.unwrap().fitness >= num {
                    return prev_ind.unwrap();
                } 

            prev_ind = next_ind;
            next_ind = pop_iter.next();
        }

        // If we don't find a closest individual, we return the first 
        // individiual in our population.
        return self.population.iter().next().unwrap();
    }
}