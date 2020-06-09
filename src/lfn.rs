pub type LevelFileName = arrayvec::ArrayString<[u8; 12]>;

pub trait FmtLevelFileName {
    fn from_fmt(pre: &str, pad: usize, num: i32) -> Self;
}

impl FmtLevelFileName for LevelFileName {
    fn from_fmt(pre: &str, pad: usize, num: i32) -> Self {
        // todo String defeats the whole point :!:!L@)(!)IJFKJFKE!! :) :) :) :) 
        let mut s = String::new();
        let num = format!("{}", num);
        s.push_str(pre);
        for _ in 0..pad - num.len() {
            s.push('0');
        }
        s.push_str(&num);
        s.push_str(".lev");
        
        LevelFileName::from(&s).unwrap()
    }
}