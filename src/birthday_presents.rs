use rand::seq::SliceRandom;
use std::collections::LinkedList;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::spawn;

enum ServantAction {
    /// Take a present from the bag and add it to the chain in the correct location
    AddPresentToChain,

    /// Remove a present from the chain and write a thank you card to the guest
    /// who gave the present.
    WriteThankYouCard,

    /// Check if a present with a given ID is on the chain or not.
    CheckIfPresentOnChain(usize),
}

// Notes
// - Each servant needs to alternate between adding a gift and writing a thank you card
// - The servants should only stop when the bag and chain are both empty

const BAG_SIZE: usize = 500000;

fn main() {
    // "Initially all of the presents were thrown into a large bag with no particular order."
    let mut large_bag = Vec::with_capacity(BAG_SIZE);

    for i in 1..BAG_SIZE + 1 {
        large_bag.push(i);
    }

    // Mix up the bag
    let mut rng = rand::thread_rng();
    large_bag.shuffle(&mut rng);

    let large_bag = Arc::new(Mutex::new(large_bag));

    let chain_of_presents = Arc::new(RwLock::new(LinkedList::new()));

    // Should be equal to BAG_SIZE when the servants are finished
    let thank_you_counter = Arc::new(AtomicU64::new(0));

    // Spawn the 4 servant threads
    let mut servant_handles = Vec::new();

    for _ in 0..4 {
        let local_bag = large_bag.clone();
        let local_chain = chain_of_presents.clone();
        let local_counter = thank_you_counter.clone();

        let join_handle = spawn(move || {
            let mut current_action = ServantAction::AddPresentToChain;

            loop {
                // Set the next action for the servant based on what the servant just did
                current_action = match current_action {
                    ServantAction::AddPresentToChain => ServantAction::WriteThankYouCard,
                    ServantAction::WriteThankYouCard => ServantAction::AddPresentToChain,
                    ServantAction::CheckIfPresentOnChain(_) => ServantAction::AddPresentToChain,
                };

                match current_action {
                    ServantAction::AddPresentToChain => {
                        let mut bag = local_bag.lock().unwrap();
                        let maybe_present = bag.pop();
                        drop(bag);

                        let present_to_add = if let Some(present) = maybe_present {
                            present
                        } else {
                            // If the bag is empty check to see if the chain is empty as well. If it is then the
                            // servant's job is done and it can return.
                            let chain = local_chain.read().unwrap();
                            let is_empty = chain.is_empty();
                            drop(chain);

                            if is_empty {
                                return;
                            } else {
                                continue;
                            }
                        };

                        let mut chain = local_chain.write().unwrap();
                        // chain.push_back(present_to_add);
                        add_present_to_chain(&mut chain, present_to_add);
                        drop(chain);
                    }
                    ServantAction::WriteThankYouCard => {
                        let mut chain = local_chain.write().unwrap();
                        let maybe_present = chain.pop_front();

                        if let None = maybe_present {
                            // If the chain is empty check to see if the bag is empty as well. If it is then the
                            // servant's job is done and it can return.
                            let bag = local_bag.lock().unwrap();
                            let is_empty = bag.is_empty();
                            drop(bag);

                            if is_empty {
                                return;
                            } else {
                                continue;
                            }
                        }

                        // Writing a thank you card is represented as adding 1 to the thank you counter
                        local_counter.fetch_add(1, Ordering::Relaxed);
                    }
                    ServantAction::CheckIfPresentOnChain(present_id) => {
                        let chain = local_chain.read().unwrap();
                        let on_chain = chain.iter().find(|x| **x == present_id).is_some();

                        drop(chain);

                        if on_chain {
                            println!("The present with ID {} is on the chain", present_id);
                        } else {
                            println!("The present with ID {} is not on the chain", present_id);
                        }
                    }
                }
            }
        });

        servant_handles.push(join_handle);
    }

    // Wait for the servants to finish
    for servant_handle in servant_handles {
        servant_handle.join().unwrap();
    }

    let final_counter = thank_you_counter.load(Ordering::Relaxed);

    println!(
        "The servants have processed {} presents and written {} thank you notes",
        BAG_SIZE, final_counter
    );
}

fn add_present_to_chain(chain: &mut LinkedList<usize>, present: usize) {
    let mut insertion_index = None;

    // Find the position of the present to add
    for (index, &item) in chain.iter().enumerate() {
        if present < item {
            insertion_index = Some(index);
            break;
        }
    }

    match insertion_index {
        Some(index) => {
            // Split the list & insert at the right position
            let mut split = chain.split_off(index);
            chain.push_back(present);
            chain.append(&mut split);
        }
        None => chain.push_back(present),
    }
}

// I'm not sure if this is necessary - the servants always just pick the present at
// the front of the chain to write a thank you note for
// fn remove_present_from_chain(chain: &mut LinkedList<usize>, present: usize) {
//     let mut maybe_removal_index = None;

//     // Find the position of the present to remove
//     for (index, &item) in chain.iter().enumerate() {
//         if present == item {
//             maybe_removal_index = Some(index);
//             break;
//         }
//     }

//     if let Some(removal_index) = maybe_removal_index {
//         let mut split = chain.split_off(removal_index);
//         split.pop_front(); // This will removes the present
//         chain.append(&mut split);
//     }
// }
