extern crate abomonation;
extern crate timely;

pub mod operators;

use timely::ExchangeData;

pub type Timestamp = u64;
pub type TraceId = u32;
pub type Degree = u32;

/// A sessionizable message.
///
/// Sessionizion requires two properties for each recorded message:
///
///    - a session identifier
///    - the log record timestamp
pub trait SessionizableMessage: ExchangeData {
//    type Timestamp: ExchangeData;

    fn time(&self) -> Timestamp;
    fn session(&self) -> &str;
}

pub trait TracedMessage: ExchangeData {
    fn call_trace(&self) -> Vec<TraceId>;
}

#[derive(Debug, Clone)]
pub struct MessagesForSession<M: SessionizableMessage> {
    pub session: String,
    pub messages: Vec<M>,
}

impl<M: SessionizableMessage + TracedMessage> MessagesForSession<M> {
        pub fn reconstruct_trace_tree(&self) -> Vec<Degree> {
            let paths: Vec<Vec<TraceId>> = self.messages
                .iter().map(|m| m.call_trace()).collect();
            let mut position = vec![0; paths.len()];
            let mut degrees = vec![0];
            let mut offsets = vec![1]; // where do children start?

            if let Some(max_depth) = paths.iter().map(|p| p.len()).max() {
                for depth in 0 .. max_depth {
                    // advance each position based on its offset
                    // ensure that the max degree of the associated node is at least as high as it should be.
                    for index in 0..paths.len() {
                        if paths[index].len() > depth {
                            if depth > 0 {
                                position[index] = (offsets[position[index]] + paths[index][depth-1]) as usize;
                            }

                            degrees[position[index]] = ::std::cmp::max(degrees[position[index]], paths[index][depth] + 1);
                        }
                    }

                    // add zeros and transform degrees to offsets.
                    let mut last = 0;
                    for &x in &degrees { last += x as usize; }

                    while degrees.len() <= last {
                        degrees.push(0);
                        offsets.push(0);
                    }

                    for i in 1..degrees.len() {
                        offsets[i] = offsets[i-1] + degrees[i-1];
                    }

                }
            }

            return degrees;
    }
}