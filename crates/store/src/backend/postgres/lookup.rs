/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::store::{IntoRows, QueryResult, QueryType, Value, backend::postgres::into_pool_error};

use bytes::BytesMut;
use futures::{TryStreamExt, pin_mut};
use tokio_postgres::types::{FromSql, ToSql, Type};

use super::{PostgresStore, into_error};

impl PostgresStore {
    pub(crate) async fn sql_query<T: QueryResult>(
        &self,
        query: &str,
        params_: &[Value<'_>],
    ) -> crate::trc::Result<T> {
        let conn = self.conn_pool.get().await.map_err(into_pool_error)?;
        let s = conn.prepare_cached(query).await.map_err(into_error)?;
        let params = params_
            .iter()
            .map(|v| v as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect::<Vec<_>>();

        match T::query_type() {
            QueryType::Execute => conn
                .execute(&s, params.as_slice())
                .await
                .map_or_else(|e| Err(into_error(e)), |r| Ok(T::from_exec(r as usize))),
            QueryType::Exists => {
                let rows = conn
                    .query_raw(&s, params.into_iter())
                    .await
                    .map_err(into_error)?;
                pin_mut!(rows);
                rows.try_next()
                    .await
                    .map_or_else(|e| Err(into_error(e)), |r| Ok(T::from_exists(r.is_some())))
            }
            QueryType::QueryOne => conn
                .query_opt(&s, params.as_slice())
                .await
                .map_or_else(|e| Err(into_error(e)), |r| Ok(T::from_query_one(r))),
            QueryType::QueryAll => conn
                .query(&s, params.as_slice())
                .await
                .map_or_else(|e| Err(into_error(e)), |r| Ok(T::from_query_all(r))),
        }
    }
}

impl ToSql for Value<'_> {
    fn to_sql(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        match self {
            Value::Integer(v) => match *ty {
                Type::CHAR => (*v as i8).to_sql(ty, out),
                Type::INT2 => (*v as i16).to_sql(ty, out),
                Type::INT4 => (*v as i32).to_sql(ty, out),
                _ => v.to_sql(ty, out),
            },
            Value::Bool(v) => v.to_sql(ty, out),
            Value::Float(v) => {
                if matches!(ty, &Type::FLOAT4) {
                    (*v as f32).to_sql(ty, out)
                } else {
                    v.to_sql(ty, out)
                }
            }
            Value::Text(v) => v.to_sql(ty, out),
            Value::Blob(v) => v.to_sql(ty, out),
            Value::Null => None::<String>.to_sql(ty, out),
        }
    }

    fn accepts(_: &tokio_postgres::types::Type) -> bool
    where
        Self: Sized,
    {
        true
    }

    fn to_sql_checked(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            Value::Integer(v) => match *ty {
                Type::CHAR => (*v as i8).to_sql_checked(ty, out),
                Type::INT2 => (*v as i16).to_sql_checked(ty, out),
                Type::INT4 => (*v as i32).to_sql_checked(ty, out),
                _ => v.to_sql_checked(ty, out),
            },
            Value::Bool(v) => v.to_sql_checked(ty, out),
            Value::Float(v) => {
                if matches!(ty, &Type::FLOAT4) {
                    (*v as f32).to_sql_checked(ty, out)
                } else {
                    v.to_sql_checked(ty, out)
                }
            }
            Value::Text(v) => v.to_sql_checked(ty, out),
            Value::Blob(v) => v.to_sql_checked(ty, out),
            Value::Null => None::<String>.to_sql_checked(ty, out),
        }
    }
}

impl IntoRows for Vec<tokio_postgres::Row> {
    fn into_rows(self) -> crate::store::Rows {
        crate::store::Rows {
            rows: self
                .into_iter()
                .map(|r| crate::store::Row {
                    values: (0..r.len())
                        .map(|idx| r.try_get(idx).unwrap_or(Value::Null))
                        .collect(),
                })
                .collect(),
        }
    }

    fn into_named_rows(self) -> crate::store::NamedRows {
        crate::store::NamedRows {
            names: self
                .first()
                .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect())
                .unwrap_or_default(),
            rows: self
                .into_iter()
                .map(|r| crate::store::Row {
                    values: (0..r.len())
                        .map(|idx| r.try_get(idx).unwrap_or(Value::Null))
                        .collect(),
                })
                .collect(),
        }
    }

    fn into_row(self) -> Option<crate::store::Row> {
        unreachable!()
    }
}

impl IntoRows for Option<tokio_postgres::Row> {
    fn into_row(self) -> Option<crate::store::Row> {
        self.map(|row| crate::store::Row {
            values: (0..row.len())
                .map(|idx| row.try_get(idx).unwrap_or(Value::Null))
                .collect(),
        })
    }

    fn into_rows(self) -> crate::store::Rows {
        unreachable!()
    }

    fn into_named_rows(self) -> crate::store::NamedRows {
        unreachable!()
    }
}

impl FromSql<'_> for Value<'static> {
    fn from_sql(
        ty: &tokio_postgres::types::Type,
        raw: &'_ [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        match ty {
            &Type::VARCHAR | &Type::TEXT | &Type::BPCHAR | &Type::NAME | &Type::UNKNOWN => {
                String::from_sql(ty, raw).map(|s| Value::Text(s.into()))
            }
            &Type::BOOL => bool::from_sql(ty, raw).map(Value::Bool),
            &Type::CHAR => i8::from_sql(ty, raw).map(|v| Value::Integer(v as i64)),
            &Type::INT2 => i16::from_sql(ty, raw).map(|v| Value::Integer(v as i64)),
            &Type::INT4 => i32::from_sql(ty, raw).map(|v| Value::Integer(v as i64)),
            &Type::INT8 | &Type::OID => i64::from_sql(ty, raw).map(Value::Integer),
            &Type::FLOAT4 | &Type::FLOAT8 => f64::from_sql(ty, raw).map(Value::Float),
            ty if (ty.name() == "citext"
                || ty.name() == "ltree"
                || ty.name() == "lquery"
                || ty.name() == "ltxtquery") =>
            {
                String::from_sql(ty, raw).map(|s| Value::Text(s.into()))
            }
            _ => Vec::<u8>::from_sql(ty, raw).map(|b| Value::Blob(b.into())),
        }
    }

    fn accepts(_: &tokio_postgres::types::Type) -> bool {
        true
    }
}
