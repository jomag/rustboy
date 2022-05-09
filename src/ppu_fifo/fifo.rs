const FIFO_SIZE: usize = 16;

pub struct FIFO {
    queue: [(u8, u8, u8, bool); FIFO_SIZE + 1],
    pub head: usize,
    pub tail: usize,
}

impl FIFO {
    pub fn new() -> Self {
        FIFO {
            queue: [(0, 0, 0, false); FIFO_SIZE + 1],
            head: 0,
            tail: 0,
        }
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
    }

    pub fn enqueue(&mut self, color: u8, prio: u8, obj_prio: u8, bg_prio: bool) {
        self.queue[self.head] = (color, prio, obj_prio, bg_prio);
        self.head += 1;
        if self.head == FIFO_SIZE + 1 {
            self.head = 0;
        }

        if self.head == self.tail {
            panic!("Pixel FIFO overrun")
        }
    }

    pub fn dequeue(&mut self) -> (u8, u8, u8, bool) {
        if self.head == self.tail {
            panic!("Pixel FIFO is empty");
        }

        let p = self.queue[self.tail];
        self.tail += 1;
        if self.tail == FIFO_SIZE + 1 {
            self.tail = 0
        }

        p
    }

    pub fn len(&self) -> usize {
        if self.head < self.tail {
            FIFO_SIZE + 1 - self.tail + self.head
        } else {
            self.head - self.tail
        }
    }
}

#[cfg(test)]
mod pixel_fifo {
    use super::*;

    #[test]
    fn len_zero_after_init() {
        let fifo = FIFO::new();
        assert_eq!(fifo.len(), 0);
    }

    #[test]
    fn len_increase_after_enqueue() {
        let mut fifo = FIFO::new();
        for i in 0..16 {
            fifo.enqueue(0, 0, 0, false);
            let len = fifo.len();
            assert_eq!(len, i + 1, "len after pushing {} items: {}", i + 1, len);
        }
    }

    #[test]
    #[should_panic(expected = "Pixel FIFO overrun")]
    fn overrun() {
        let mut fifo = FIFO::new();
        // Push one more than in the previous tests
        for _ in 0..17 {
            fifo.enqueue(0, 0, 0, false);
        }
    }

    #[test]
    #[should_panic(expected = "Pixel FIFO is empty")]
    fn dequeue_when_empty() {
        let mut fifo = FIFO::new();
        fifo.dequeue();
    }

    #[test]
    fn len_decrease_when_dequeued() {
        let mut fifo = FIFO::new();
        for _ in 0..3 {
            fifo.enqueue(0, 0, 0, false);
        }
        assert_eq!(fifo.len(), 3);
        for i in 0..3 {
            fifo.dequeue();
            assert_eq!(fifo.len(), 3 - 1 - i);
        }
        assert_eq!(fifo.len(), 0);
    }

    #[test]
    fn iterate() {
        let mut fifo = FIFO::new();
        for _ in 0..100 {
            assert_eq!(fifo.len(), 0);
            fifo.enqueue(0, 0, 0, false);
            assert_eq!(fifo.len(), 1);
            fifo.dequeue();
        }
    }
}
