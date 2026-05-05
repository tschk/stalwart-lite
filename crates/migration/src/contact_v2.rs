/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::common::{DavName, Server};
use crate::groupware::contact::ContactCard;
use crate::store::{
    Serialize, ValueKey,
    write::{AlignedBytes, Archive, Archiver, BatchBuilder, serialize::rkyv_deserialize},
};
use crate::trc::AddContext;
use crate::types::{collection::Collection, dead_property::DeadProperty, field::Field};

use crate::migration::get_document_ids;

#[derive(
    rkyv::Archive, rkyv::Deserialize, rkyv::Serialize, Debug, Default, Clone, PartialEq, Eq,
)]
pub struct ContactCardV2 {
    pub names: Vec<DavName>,
    pub display_name: Option<String>,
    pub card: calcard_v01::vcard::VCard,
    pub dead_properties: DeadProperty,
    pub created: i64,
    pub modified: i64,
    pub size: u32,
}

pub(crate) async fn migrate_contacts_v013(
    server: &Server,
    account_id: u32,
) -> crate::trc::Result<u64> {
    let document_ids = get_document_ids(server, account_id, Collection::ContactCard)
        .await
        .caused_by(crate::trc::location!())?
        .unwrap_or_default();

    let mut num_migrated = 0;

    for document_id in document_ids.iter() {
        let Some(archive) = server
            .store()
            .get_value::<Archive<AlignedBytes>>(ValueKey::archive(
                account_id,
                Collection::ContactCard,
                document_id,
            ))
            .await
            .caused_by(crate::trc::location!())?
        else {
            continue;
        };

        match archive.unarchive_untrusted::<ContactCardV2>() {
            Ok(contact) => {
                let contact = rkyv_deserialize::<_, ContactCardV2>(contact).unwrap();
                let new_contact = ContactCard {
                    names: contact.names,
                    display_name: contact.display_name,
                    dead_properties: contact.dead_properties,
                    size: contact.size,
                    created: contact.created,
                    modified: contact.modified,
                    card: calcard::vcard::VCard::parse(contact.card.to_string())
                        .unwrap_or_default(),
                };

                let mut batch = BatchBuilder::new();
                batch
                    .with_account_id(account_id)
                    .with_collection(Collection::ContactCard)
                    .with_document(document_id)
                    .set(
                        Field::ARCHIVE,
                        Archiver::new(new_contact)
                            .serialize()
                            .caused_by(crate::trc::location!())?,
                    );
                server
                    .store()
                    .write(batch.build_all())
                    .await
                    .caused_by(crate::trc::location!())?;
                num_migrated += 1;
            }
            Err(err) => {
                if let Err(err_) = archive.unarchive_untrusted::<ContactCard>() {
                    crate::trc::error!(err_.caused_by(crate::trc::location!()));
                    return Err(err.caused_by(crate::trc::location!()));
                }
            }
        }
    }

    Ok(num_migrated)
}
