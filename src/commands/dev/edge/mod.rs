mod server;
mod setup;

use server::serve;
use setup::Init;

use crate::commands::dev::{socket, ServerConfig};
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};

use tokio::runtime::Runtime as TokioRuntime;

pub fn dev(
    target: Target,
    deploy_config: DeployConfig,
    user: GlobalUser,
    server_config: ServerConfig,
    verbose: bool,
) -> Result<(), failure::Error> {
    let init = Init::new(&target, &deploy_config, &user)?;
    let mut target = target;

    // TODO: replace asset manifest parameter
    let preview_token = setup::upload(
        &mut target,
        &deploy_config,
        &user,
        init.preview_token.clone(),
        verbose,
    )?;

    let mut runtime = TokioRuntime::new()?;
    runtime.block_on(async {
        let devtools_listener = tokio::spawn(socket::listen(init.websocket_url));
        let server = tokio::spawn(serve(server_config, preview_token, init.host));
        let res = tokio::try_join!(async { devtools_listener.await? }, async { server.await? });

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    })
}
