use crate::runtime::CoLocatedRuntime;

#[derive(Clone)]
pub struct AppState {
    pub runtime: CoLocatedRuntime,
}
