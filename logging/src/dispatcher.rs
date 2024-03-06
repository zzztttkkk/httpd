use std::marker::PhantomData;

use crate::channel::Channel;

pub struct Dispatcher<C: Channel> {
    _c: PhantomData<C>,
}
