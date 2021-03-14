pub struct Results {
    pub(crate) win: u32,
    pub(crate) lose: u32,
}

impl Results {
    pub(crate) fn new() -> Self {
        Results {
            win: 0,
            lose: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.win = 0;
        self.lose = 0;
    }
}

