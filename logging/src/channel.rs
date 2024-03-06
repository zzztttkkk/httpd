pub trait Channel: Send + Sync + Sized + Clone + Copy {
    fn idx(&self) -> u8;
}

pub(crate) struct ChannelStorage {}

#[cfg(test)]
mod tests {
    use super::Channel;

    #[derive(Clone, Copy, Debug)]
    enum MyChannel {
        A,
        B,
    }

    impl Channel for MyChannel {
        fn idx(&self) -> u8 {
            match self {
                MyChannel::A => 0,
                MyChannel::B => 1,
            }
        }
    }
}
