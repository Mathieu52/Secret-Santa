use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use rand::Rng;
use rand::seq::SliceRandom;
use itertools::Itertools;
use timing::Timer;

mod secret_santa;
mod ui;
mod participant;

use secret_santa::{generate_secret_santa};
use participant::Participant;

fn generate_participants<'a>(number: usize) -> HashSet<Participant> {
    let mut participants = HashSet::with_capacity(number);

    for i in 1..number + 1 {
        participants.insert(Participant {name: format!("Participant {}", i)});
    }

    participants
}

fn generate_large_exclusions<'a>(
    participants: &HashSet<&'a Participant>,
    exclusion_probability: f64,
) -> HashMap<&'a Participant, HashSet<&'a Participant>> {
    let mut rng = rand::thread_rng();
    let mut exclusions: HashMap<&'a Participant, HashSet<&'a Participant>> = HashMap::new();

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
macro_rules! time_exec {
    ($label:expr, $body:block) => {
        {
            let timer = Timer::with_label($label);
            println!("------------------------------------------------------------------");
            println!("{}: Starting execution", $label);
            // Execute the body (the code passed to the macro)
            let result = $body;
            println!("{}: Execution finished in {:?}", $label, timer.stop());
            println!("------------------------------------------------------------------");

            result
        }
    };
}
fn main() {
    let binding: HashSet<_> = time_exec!("Binding generation", {
        generate_participants(5000)
    });
    let participants: HashSet<_> = time_exec!("Participant generation", {
        binding.iter().map(|s| s).collect()
    });
    let mut exclusions = time_exec!("Exclusions generation", {
        generate_large_exclusions(&participants, 0.9)
    });

    let results = time_exec!("Paring", {
        generate_secret_santa(&participants, &mut exclusions)
    });

    match results {
        Some(pairings) => {
            for (giver, receiver) in pairings {
                println!("{} gives a gift to {}", giver, receiver);
            }
            println!("Valid Secret Santa assignment found.");
        }
        None => println!("No valid Secret Santa assignment found."),
    }
}