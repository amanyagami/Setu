//! Priority Scheduler per Setu v5 §9.3
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FramePriority {
    KeyFrame = 0,
    DeltaImportant = 1,
    DeltaNormal = 2,
    Metadata = 3,
}

pub struct PriorityScheduler {
    queues: Vec<Vec<Vec<u8>>>,
}

impl PriorityScheduler {
    pub fn new() -> Self {
        Self { queues: vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()] }
    }
    
    pub fn enqueue(&mut self, data: Vec<u8>, priority: FramePriority) {
        let idx = priority as usize;
        self.queues[idx].push(data);
    }
    
    pub fn dequeue(&mut self) -> Option<Vec<u8>> {
        for queue in &mut self.queues {
            if !queue.is_empty() {
                return Some(queue.remove(0));
            }
        }
        None
    }
}

impl Default for PriorityScheduler {
    fn default() -> Self { Self::new() }
}
