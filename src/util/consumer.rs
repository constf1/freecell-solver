/// Represents an operation that accepts a single input argument and produces a result.
/// Should return [`true`] to continue or [`false`] to stop.
/// Consumer is expected to operate via side-effects.
pub trait Consumer<T> {
    /// Performs this operation on the given argument.
    /// `t` - the input argument
    fn accept(&mut self, t: T) -> bool;
}

/// Single operation consumer.
pub struct SingleConsumer<T>(Option<T>);
/// Consumes all operations into a [`Vec`].
pub struct TotalConsumer<T>(Vec<T>);

impl<T> SingleConsumer<T> {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn take_one<F>(func: F) -> Option<T>
    where
        F: FnOnce(&mut Self),
    {
        Self::new().apply(func)
    }

    pub fn once<F>(func: F) -> bool
    where
        F: FnOnce(&mut Self),
    {
        Self::take_one(func).is_some()
    }

    pub fn apply<F>(&mut self, func: F) -> Option<T>
    where
        F: FnOnce(&mut Self),
    {
        func(self);
        self.take()
    }

    pub fn take(&mut self) -> Option<T> {
        self.0.take()
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }
}

impl<T> Consumer<T> for SingleConsumer<T> {
    fn accept(&mut self, t: T) -> bool {
        if self.0.is_none() {
            self.0.replace(t);
        }
        false
    }
}

impl<T> TotalConsumer<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> Consumer<T> for TotalConsumer<T> {
    fn accept(&mut self, t: T) -> bool {
        self.0.push(t);
        true
    }
}
