pub enum ButtonType {
    Up = 64,
    Down = 128,
    Left = 32,
    Right = 16,
    Select = 4,
    Start = 8,
    A = 1,
    B = 2,
}

pub struct Buttons {
    button_state: u8,
    p1: u8,
    pub irq: u8,
}

impl Buttons {
    pub fn new() -> Self {
        Buttons {
            button_state: 0xff,
            p1: 0xff,
            irq: 0,
        }
    }

    pub fn handle_press(&mut self, btn: ButtonType) {
        self.button_state = self.button_state & !(btn as u8);
        self.update();
        // println!("Handle Press! {:x} {:x}", self.p1, self.button_state);
    }

    pub fn handle_release(&mut self, btn: ButtonType) {
        self.button_state = self.button_state | btn as u8;
        self.update();
        // println!("Handle Release! {:x} {:x}", self.p1, self.button_state);
    }

    pub fn write_p1(&mut self, v: u8) {
        self.p1 = 0xC0 | (v & 0x30) | (self.p1 & 0xF);
    }

    pub fn read_p1(&self) -> u8 {
        return self.p1;
    }

    pub fn update(&mut self) {
        let mut next = self.p1 & 0xF0;

        if self.p1 & 0x10 != 0 {
            next = next | self.button_state & 0x0F;
        }

        if self.p1 & 0x20 != 0 {
            next = next | (self.button_state >> 4) & 0x0F;
        }

        self.p1 = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const P14_MASK: u8 = 1 << 4;
    const P15_MASK: u8 = 1 << 5;
    const SELECT_OR_UP_MASK: u8 = 4;

    #[test]
    fn test_initial_state() {
        let b = Buttons::new();
        assert_eq!(b.read_p1(), 0xFF)
    }

    #[test]
    fn test_up_button() {
        let mut btn = Buttons::new();
        btn.write_p1(P15_MASK);
        btn.handle_press(ButtonType::Up);
        btn.update();
        assert!(btn.read_p1() & SELECT_OR_UP_MASK == 0);
        btn.handle_release(ButtonType::Up);
        btn.update();
        assert!(btn.read_p1() & SELECT_OR_UP_MASK != 0)
    }

    #[test]
    fn test_select_button() {
        let mut btn = Buttons::new();
        btn.write_p1(P14_MASK);
        btn.handle_press(ButtonType::Select);
        btn.update();
        assert!(btn.read_p1() & SELECT_OR_UP_MASK == 0);
        btn.handle_release(ButtonType::Select);
        btn.update();
        assert!(btn.read_p1() & SELECT_OR_UP_MASK != 0)
    }
}
