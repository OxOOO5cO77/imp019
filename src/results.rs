pub struct Results {
    pub win: u8,
    pub lose: u8,
}

impl Results {
    pub fn new() -> Self {
        Results {
            win: 0,
            lose: 0,
        }
    }

    pub fn reset(&mut self) {
        self.win = 0;
        self.lose = 0;
    }
}

