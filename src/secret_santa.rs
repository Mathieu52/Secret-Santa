use rand::seq::{IteratorRandom};
use std::collections::{HashMap, HashSet};
use itertools::Itertools;
use crate::generate_large_exclusions;

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

pub(crate) fn generate_secret_santa2<'a>(
    participants: &HashSet<&'a str>,
    exclusions: &mut HashMap<&'a str, HashSet<&'a str>>,
) -> Option<HashMap<&'a str, &'a str>> {
    let mut rng = rand::thread_rng();
    let mut exclusion_graph: HashMap<&'a str, HashSet<&'a str>> = exclusions.iter().map(|(&key, value)| (key, value.clone())).collect();
    let mut stack = Vec::with_capacity(participants.len());

    let mut unassigned = participants.clone();

    // Choose a random starting participant
    let start_participant = *participants.iter().max_by_key(|&&participant| exclusion_graph.get(participant).map_or(0, |exclusions| exclusions.len()))?;
    stack.push(start_participant);

    unassigned.remove(start_participant);

    // Backtracking loop to build a valid cycle
    while stack.len() < participants.len() {
        println!("{}%", 100f32 * (stack.len() as f32 / participants.len() as f32));

        // Precompute available participants to minimize redundant filter checks
        let giver = *stack.last()?;
        let excluded = exclusion_graph
            .entry(giver)
            .or_insert_with(HashSet::new);

        // Collect all non-excluded, non-visited participants
        let available: Vec<_> = unassigned
            .iter()
            .filter(|&&p| p != giver && !excluded.contains(p))
            .cloned()
            .collect();

        if let Some(recipient) = available.iter().choose(&mut rng).cloned() {
            stack.push(recipient);
            unassigned.remove(recipient);
        } else {
            // Backtrack if no valid recipient found
            let last_giver = stack.pop()?;
            unassigned.insert(last_giver);

            // Restore exclusions for the backtracked participant
            for exclusions in exclusion_graph.values_mut() {
                exclusions.insert(giver);
            }
        }
    }

    // Create pairs and close the cycle
    let secret_santa_pairs = stack
        .iter()
        .cloned()
        .tuple_windows()
        .collect::<HashMap<_, _>>()
        .into_iter()
        .chain(
            stack.first().zip(stack.last()).map(|(&first, &last)| (last, first)),
        )
        .collect::<HashMap<_, _>>();

    Some(secret_santa_pairs)
}


/// Generates a Secret Santa pairing, ensuring exclusions are respected.
pub(crate) fn generate_secret_santa<'a>(
    participants: &HashSet<&'a str>,
    exclusions: &HashMap<&'a str, HashSet<&'a str>>,
) -> Option<HashMap<&'a str, &'a str>> {
    let mut rng = rand::thread_rng();
    let mut participants_graph = exclusions_to_adjacency(participants, exclusions);
    let mut stack = Vec::with_capacity(participants.len());

    // Choose a random starting participant
    let start_participant = participants.iter().cloned().choose(&mut rng)?;
    stack.push(start_participant);

    // Backtracking loop to build a valid cycle
    while stack.len() < participants.len() {
        let &giver = stack.last()?;
        if let Some(recipient) = participants_graph
            .get_mut(giver)
            .and_then(|recipients| recipients.iter().choose(&mut rng).cloned())
        {
            stack.push(recipient);
            participants_graph.values_mut().for_each(|recipients| {
                recipients.remove(&recipient);
            });
        } else {
            // Backtrack if no valid recipient found
            let last_giver = stack.pop()?;
            participants_graph
                .get_mut(last_giver)
                .map(|recipients| recipients.remove(giver));
        }
    }

    // Create pairs and close the cycle
    stack.iter().cloned().tuple_windows().collect::<HashMap<_, _>>().into_iter().chain(
        stack.first().zip(stack.last()).map(|(&first, &last)| (last, first)),
    ).collect::<HashMap<_, _>>().into()
}