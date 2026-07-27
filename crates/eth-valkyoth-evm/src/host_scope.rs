pub(super) trait PoisonScopeHost {
    fn poison_scope(&mut self);
}

pub(super) struct PoisonScope<'scope, H: PoisonScopeHost> {
    host: &'scope mut H,
    finalized: bool,
}

impl<'scope, H: PoisonScopeHost> PoisonScope<'scope, H> {
    pub(super) fn new(host: &'scope mut H) -> Self {
        Self {
            host,
            finalized: false,
        }
    }

    pub(super) fn host(&mut self) -> &mut H {
        self.host
    }

    pub(super) fn finish(mut self) {
        self.finalized = true;
    }
}

impl<H: PoisonScopeHost> Drop for PoisonScope<'_, H> {
    fn drop(&mut self) {
        if !self.finalized {
            self.host.poison_scope();
        }
    }
}
