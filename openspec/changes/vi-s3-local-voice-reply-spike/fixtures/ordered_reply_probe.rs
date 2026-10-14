use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outcome {
    Ready,
    Skipped,
}

#[derive(Debug, PartialEq, Eq)]
enum AcceptError {
    CapacityExceeded,
}

struct OrderedReplyBuffer {
    generation: u64,
    next_sequence: u64,
    capacity: usize,
    buffered: BTreeMap<u64, Outcome>,
}

impl OrderedReplyBuffer {
    fn new(generation: u64, capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            generation,
            next_sequence: 0,
            capacity,
            buffered: BTreeMap::new(),
        }
    }

    fn begin_generation(&mut self, generation: u64) {
        self.generation = generation;
        self.next_sequence = 0;
        self.buffered.clear();
    }

    fn accept(
        &mut self,
        generation: u64,
        sequence: u64,
        outcome: Outcome,
    ) -> Result<Vec<(u64, Outcome)>, AcceptError> {
        if generation != self.generation || sequence < self.next_sequence {
            return Ok(Vec::new());
        }
        if sequence == self.next_sequence {
            let mut released = vec![(sequence, outcome)];
            self.next_sequence += 1;
            while let Some(value) = self.buffered.remove(&self.next_sequence) {
                released.push((self.next_sequence, value));
                self.next_sequence += 1;
            }
            return Ok(released);
        }
        if self.buffered.contains_key(&sequence) {
            return Ok(Vec::new());
        }
        if self.buffered.len() == self.capacity {
            return Err(AcceptError::CapacityExceeded);
        }
        self.buffered.insert(sequence, outcome);
        Ok(Vec::new())
    }
}

fn main() {
    let mut buffer = OrderedReplyBuffer::new(7, 3);
    assert!(buffer.accept(7, 2, Outcome::Ready).unwrap().is_empty());
    assert!(buffer.accept(7, 1, Outcome::Skipped).unwrap().is_empty());
    assert_eq!(
        buffer.accept(7, 0, Outcome::Ready).unwrap(),
        vec![
            (0, Outcome::Ready),
            (1, Outcome::Skipped),
            (2, Outcome::Ready)
        ]
    );

    buffer.begin_generation(8);
    assert!(buffer.accept(7, 3, Outcome::Ready).unwrap().is_empty());
    assert_eq!(
        buffer.accept(8, 0, Outcome::Ready).unwrap(),
        vec![(0, Outcome::Ready)]
    );

    assert!(buffer.accept(8, 4, Outcome::Ready).unwrap().is_empty());
    assert!(buffer.accept(8, 3, Outcome::Ready).unwrap().is_empty());
    assert!(buffer.accept(8, 2, Outcome::Ready).unwrap().is_empty());
    assert_eq!(
        buffer.accept(8, 5, Outcome::Ready),
        Err(AcceptError::CapacityExceeded)
    );
    assert_eq!(
        buffer.accept(8, 1, Outcome::Ready).unwrap(),
        vec![
            (1, Outcome::Ready),
            (2, Outcome::Ready),
            (3, Outcome::Ready),
            (4, Outcome::Ready)
        ]
    );

    println!(
        "{{\"kind\":\"ordered-reply-contract\",\"ordered_release\":true,\"skipped_sequence_progresses\":true,\"stale_generation_rejected\":true,\"bounded_capacity\":true}}"
    );
}
