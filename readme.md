## To compile & run problem 1

```bash
cargo run --bin birthday_presents --release
```

## To compile & run problem 2

```bash
cargo run --bin temperature --release
```

## Problem 1 (birthday presents)
- I decided to use a `Arc<RwLock<std::collections::LinkedList>>` as the shared linked list. I chose an `RwLock` over a `Mutex` so multiple servants can check if a gift exists on the chain as long as there's no other servants writing to the chain. 
- I used a `Arc<Mutex<Vec>>` for the unordered bag of presents. Each present is represented as a number 1 - 500,000. The Vector is shuffled before being passed to each servant.
- I used an `Arc<AtomicU64>` as the counter for thank you notes.
- References to all three data structures are passed to all servant threads upon creation.
- I created a function `add_present_to_chain` that takes a given present and adds it into the correct position into the chain.

## Problem 2 (temperature)
- For this one I used an `mpsc`, a multi-producer, single consumer queue. The 8 sensor reporting threads act as the producer and a single shared memory report generating thread acts as the consumer.
- I decided to use a queue because the sensor threads will always be able to push onto it with no chance of blocking. 
- The report thread is also able to request temperature readings from the queue as well whenever it wants. If the report thread is busy the queue will hold all the recordings until it's ready to intake more recordings.
- The sensor threads are very simple, all they do is generate a temperature value along with a timestamp and push it onto the queue on an interval.
