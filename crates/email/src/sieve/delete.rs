/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use super::SieveScript;
use crate::common::{Server, auth::AccessToken, storage::index::ObjectIndexBuilder};
use crate::store::write::BatchBuilder;
use crate::store::{
    ValueKey,
    write::{AlignedBytes, Archive},
};
use crate::trc::AddContext;
use crate::types::{collection::Collection, field::SieveField};

pub trait SieveScriptDelete: Sync + Send {
    fn sieve_script_delete(
        &self,
        account_id: u32,
        document_id: u32,
        access_token: &AccessToken,
        batch: &mut BatchBuilder,
    ) -> impl Future<Output = crate::trc::Result<bool>> + Send;
}

impl SieveScriptDelete for Server {
    async fn sieve_script_delete(
        &self,
        account_id: u32,
        document_id: u32,
        access_token: &AccessToken,
        batch: &mut BatchBuilder,
    ) -> crate::trc::Result<bool> {
        // Fetch record
        if let Some(obj_) = self
            .store()
            .get_value::<Archive<AlignedBytes>>(ValueKey::archive(
                account_id,
                Collection::SieveScript,
                document_id,
            ))
            .await?
        {
            // Delete record
            batch
                .with_account_id(account_id)
                .with_collection(Collection::SieveScript)
                .with_document(document_id)
                .clear(SieveField::Ids)
                .custom(
                    ObjectIndexBuilder::<_, ()>::new()
                        .with_current(
                            obj_.to_unarchived::<SieveScript>()
                                .caused_by(crate::trc::location!())?,
                        )
                        .with_access_token(access_token),
                )
                .caused_by(crate::trc::location!())?
                .commit_point();

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
