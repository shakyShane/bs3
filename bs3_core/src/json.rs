use crate::browser_sync::BrowserSync;
use crate::start;

pub async fn from_json(json: String) -> Result<(), anyhow::Error> {
    actix_rt::System::new("bs3_core::from_json").block_on(async move {
        let bs = BrowserSync::try_from_json(json)?;
        let fut = start::main(bs, None);
        fut.await
    })
}
