use std::sync::mpsc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use rand::Rng;

// Notes
// 8 temperature reading threads
// Must use shared memory to store & make the reports

// Ideas
// Every minute could be treated as a millisecond
// Threads will do their operations and wait until it's time to do stuff again

// - Sensor threads will record & push onto queue every minute

// - Report thread will pull from the queue and add to the list until it's time
// to make a report. Every iteration of the report thread loop will check to see
// if it's time to make a report.

// The temp sensor threads will be very simple. They'll generate a random number,
// push it onto the queue, and wait for another minute.

// There will be 1 report thread that receives message from
// the sensors and adds them to the report.

// This is a perfect use case for a multi producer single consumer queue

// The queue won't every delay a sensor, sensors will just push onto the queue and
// continue

// The queue will hold readings until the report thread is ready to read them again
// so no readings will ever get lost.

// - All temp readings for a given hour will be stored in a list

// - When its time for a report to be generated all readings will be
// taken from the list and used to compile the report

const ONE_HOUR_MS: u64 = 3600000;
const ONE_MINUTE_MS: u64 = 60000;
const SPEEDUP_FACTOR: u64 = 250;

#[derive(Clone, Debug)]
struct Recording {
    temperature: i64,
    timestamp: Instant,
}

impl Recording {
    pub fn new() -> Recording {
        let mut rng = rand::thread_rng();
        Recording {
            temperature: rng.gen_range(-100..=70),
            timestamp: Instant::now(),
        }
    }
}

#[derive(Debug)]
struct Report {
    top_five_lowest_temps: Vec<Recording>,
    top_five_highest_temps: Vec<Recording>,
    largest_temp_difference: (Instant, Instant, i64),
}

fn main() {
    let scaled_hour = ONE_HOUR_MS / SPEEDUP_FACTOR;
    let scaled_minute = ONE_MINUTE_MS / SPEEDUP_FACTOR;

    // Enables communication from the temperature recording threads (multi producer) to the report thread (single consumer)
    let (temperature_sender, temperature_receiver) = mpsc::channel::<Recording>();

    for _ in 0..8 {
        let local_sender = temperature_sender.clone();

        std::thread::spawn(move || loop {
            local_sender.send(Recording::new()).unwrap();

            let time_now = Instant::now();
            let wake_up_at = time_now + Duration::from_millis(scaled_minute);
            let duration_to_sleep = wake_up_at - time_now;
            sleep(duration_to_sleep);
        });
    }

    println!("The sensor threads have been created and are pushing recordings onto the queue");

    // The temperature receiving & report making process is done in a separate thread but it
    // can easily be done in the main thread as well.
    let report_thread_join_handle = std::thread::spawn(move || {
        let mut last_report_generated = Instant::now();

        let mut generate_next_report_at =
            last_report_generated + Duration::from_millis(scaled_hour);

        let mut recordings = vec![];

        loop {
            if Instant::now() > generate_next_report_at {
                // Take all the values from recordings and put them into report_recordings
                let mut report_recordings: Vec<Recording> = recordings.drain(..).collect();

                // Sort the recordings by temperature and record the lowest & highest temps
                report_recordings.sort_by_key(|x| x.temperature);

                let top_five_lowest_temps: Vec<Recording> =
                    report_recordings.iter().take(5).cloned().collect();

                let top_five_highest_temps: Vec<Recording> =
                    report_recordings.iter().rev().take(5).cloned().collect();

                // Sort the recordings by timestamp and find the interval in which the largest temp difference was observed
                report_recordings.sort_by_key(|x| x.timestamp);

                let largest_temp_difference =
                    if let Some(x) = find_largest_temp_difference(&report_recordings) {
                        x
                    } else {
                        println!("No recordings available to compare, report thread returning");
                        return;
                    };

                println!("\nA new report has been generated\n");

                println!("Top 5 lowest temps: ");
                for recording in top_five_lowest_temps.iter() {
                    print!("{}, ", recording.temperature);
                }
                println!("\n");

                println!("Top 5 highest temps: ");
                for recording in top_five_highest_temps.iter() {
                    print!("{}, ", recording.temperature);
                }
                println!("\n");

                println!(
                    "Largest temperature difference: {}",
                    largest_temp_difference.2
                );

                // let report = Report {
                //     top_five_lowest_temps,
                //     top_five_highest_temps,
                //     largest_temp_difference,
                // };

                last_report_generated = Instant::now();
                generate_next_report_at =
                    last_report_generated + Duration::from_millis(scaled_hour);
            }

            // This reporting thread shouldn't wait forever for a new recording.
            // If there's no new recording received in one minut it'll check to see if a report should be generated
            let maybe_recording =
                temperature_receiver.recv_timeout(Duration::from_millis(scaled_minute));

            if let Ok(recording) = maybe_recording {
                recordings.push(recording);
            }
        }
    });

    println!("The report thread has been created and is processing recordings from the queue");

    report_thread_join_handle.join().unwrap();
}

// Compares every recording against every other recording. Skips the comparison if the recording isn't within
// 10 minutes.
fn find_largest_temp_difference(recordings: &Vec<Recording>) -> Option<(Instant, Instant, i64)> {
    let interval = Duration::from_millis((ONE_MINUTE_MS * 10) / SPEEDUP_FACTOR);

    let mut result: Option<(Instant, Instant, i64)> = None;

    for (index, start_rec) in recordings.iter().enumerate() {
        let start_time = start_rec.timestamp;

        for end_rec in recordings.iter().skip(index + 1) {
            let end_time = end_rec.timestamp;

            // Skip comparison if this recording isn't within 10 minutes of the other
            if end_time.duration_since(start_time) > interval {
                break;
            }

            let current_diff = (end_rec.temperature - start_rec.temperature).abs();

            // Compare against the previous largest temperature difference
            if let Some((_, _, previous_diff)) = result {
                if current_diff > previous_diff {
                    result = Some((start_time, end_time, current_diff));
                }
            } else {
                result = Some((start_time, end_time, current_diff));
            }
        }
    }

    result
}
