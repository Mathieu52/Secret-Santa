use std::collections::{HashMap, HashSet};
use rand::Rng;
use rand::seq::SliceRandom;
use itertools::Itertools;

mod secret_santa;
use secret_santa::{generate_secret_santa, generate_secret_santa2};

fn generate_participants<'a>(number: usize) -> HashSet<String> {
    let mut participants = HashSet::with_capacity(number);

    for i in 1..number + 1 {
        participants.insert(format!("Participant {}", i));
    }

    participants
}

fn generate_large_exclusions<'a>(
    participants: &HashSet<&'a str>,
    exclusion_probability: f64,
) -> HashMap<&'a str, HashSet<&'a str>> {
    let mut rng = rand::thread_rng();
    let mut exclusions: HashMap<&'a str, HashSet<&'a str>> = HashMap::new();

    // Initialize exclusions for all participants
    for participant in participants.iter() {
        exclusions.insert(*participant, HashSet::new());
    }

    // Create exclusions by iterating over pairs
    let participants_vec: Vec<_> = participants.iter().collect();
    for i in 0..participants_vec.len() {
        for j in i + 1..participants_vec.len() {
            let giver = participants_vec[i];
            let receiver = participants_vec[j];

            // Generate random exclusion decisions for both directions
            let exclusion_giver_to_receiver = rng.gen_bool(exclusion_probability);
            let exclusion_receiver_to_giver = rng.gen_bool(exclusion_probability);

            if exclusion_giver_to_receiver {
                exclusions.entry(*giver).or_insert_with(HashSet::new).insert(*receiver);
            }
            if exclusion_receiver_to_giver {
                exclusions.entry(*receiver).or_insert_with(HashSet::new).insert(*giver);
            }
        }
    }

    exclusions
}


fn main() {
    let binding = generate_participants(10000);
    let participants = binding.iter().map(|s| &**s).collect();
    let mut exclusions = generate_large_exclusions(&participants, 0.1);

    println!("Start");

    match generate_secret_santa2(&participants, &mut exclusions) {
        Some(pairings) => {
            for (giver, receiver) in pairings {
                println!("{} gives a gift to {}", giver, receiver);
            }
            println!("Valid Secret Santa assignment found.");
        }
        None => println!("No valid Secret Santa assignment found."),
    }
}
