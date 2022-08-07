fn main() {}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use workspaces::prelude::*;
    use workspaces::{Contract, DevNetwork, Worker};

    async fn init(worker: &Worker<impl DevNetwork>) -> Result<Contract> {
        let wasm = std::fs::read("../target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")?;
        let contract = worker.dev_deploy(&wasm).await?;

        return Ok(contract);
    }

    #[tokio::test]
    async fn round_trip() -> Result<()> {
        let worker = workspaces::sandbox().await?;
        let _contract = init(&worker).await?;
        Ok(())
    }
}
