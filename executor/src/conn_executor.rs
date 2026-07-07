// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

#[derive(Clone)]
pub struct ConnExecutor;

impl<F> hyper::rt::Executor<F> for ConnExecutor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}
