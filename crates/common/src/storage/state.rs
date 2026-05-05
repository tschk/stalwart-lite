/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::common::{
    IPC_CHANNEL_BUFFER, Server,
    auth::AccessToken,
    ipc::{PushEvent, PushNotification},
};
use crate::types::type_state::DataType;
use crate::utils::map::bitmap::Bitmap;
use tokio::sync::mpsc;

impl Server {
    pub async fn subscribe_push_manager(
        &self,
        access_token: &AccessToken,
        types: Bitmap<DataType>,
    ) -> crate::trc::Result<mpsc::Receiver<PushNotification>> {
        let (tx, rx) = mpsc::channel::<PushNotification>(IPC_CHANNEL_BUFFER);
        let push_tx = self.inner.ipc.push_tx.clone();

        push_tx
            .send(PushEvent::Subscribe {
                account_ids: access_token.member_ids().collect(),
                types,
                tx,
            })
            .await
            .map_err(|err| {
                crate::trc::EventType::Server(crate::trc::ServerEvent::ThreadError)
                    .reason(err)
                    .caused_by(crate::trc::location!())
            })?;

        Ok(rx)
    }
}
