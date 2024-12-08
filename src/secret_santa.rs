use rand::seq::{IteratorRandom};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use itertools::Itertools;

/// Converts exclusions to an adjacency list representation of participants and their valid recipients.
fn exclusions_to_adjacency<'a>(
    participants: &HashSet<&'a str>,
    exclusions: &HashMap<&'a str, HashSet<&'a str>>,
) -> HashMap<&'a str, HashSet<&'a str>> {
    participants
        .iter()
        .map(|&participant| {
            let valid_recipients = participants
                .iter()
                .filter(|&&p| p != participant && !exclusions.get(participant).map_or(false, |ex| ex.contains(p)))
                .cloned()
                .collect();
            (participant, valid_recipients)
        })
        .collect()
}

/// Generates a Secret Santa pairing, ensuring exclusions are respected.
pub(crate) fn generate_secret_santa<'a, T>(
    participants: &[&'a T],
    exclusions: &mut HashMap<&'a T, HashSet<&'a T>>,
) -> Option<HashMap<&'a T, &'a T>>
where
    T: Eq + Hash,
{
    let mut rng = rand::thread_rng();
    let mut exclusion_graph: HashMap<&'a T, HashSet<&'a T>> = exclusions.iter().map(|(&key, value)| (key, value.clone())).collect();
    let mut stack = Vec::with_capacity(participants.len());

    let mut unassigned = participants.clone();

    // Choose a starting participant with the most exclusions
    let start_participant = *participants
        .iter()
        .max_by_key(|&&participant| exclusion_graph.get(participant).map_or(0, |exclusions| exclusions.len()))?;
    stack.push(start_participant);

    unassigned.remove(start_participant);

    // Backtracking loop to build a valid cycle
    while stack.len() < participants.len() {
        let giver = *stack.last()?;
        let excluded = exclusion_graph.entry(giver).or_insert_with(HashSet::new);

        // Collect all non-excluded, non-visited participants
        let available: Vec<_> = unassigned
            .iter()
            .filter(|&&p| p != giver && !excluded.contains(p))
            .cloned()
            .collect();

        if let Some(&recipient) = available.iter().choose(&mut rng) {
            stack.push(recipient);
            unassigned.remove(recipient);
        } else {
            // Backtrack if no valid recipient found
            let last_giver = stack.pop()?;
            unassigned.insert(last_giver);

            // Restore exclusions for the backtracked participant
            for exclusions in exclusion_graph.values_mut() {
                exclusions.insert(last_giver);
            }
        }
    }

    // Create pairs and close the cycle
    let secret_santa_pairs = stack
        .iter()
        .zip(stack.iter().cycle().skip(1))
        .map(|(&giver, &recipient)| (giver, recipient))
        .collect();

    Some(secret_santa_pairs)
}