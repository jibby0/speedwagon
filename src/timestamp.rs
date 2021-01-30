// Custom date + time type
// Fulfills serde + diesel traits

use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types, *,
};
use serde::{
    de, de::Visitor, Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    fmt,
    io::Write,
    ops::{Add, Sub},
};

#[derive(Debug, AsExpression, FromSqlRow, PartialEq, Clone, Copy)]
#[sql_type = "sql_types::Timestamp"]
pub struct Timestamp(pub time::Timespec);

impl Timestamp {
    pub fn now() -> Timestamp {
        Timestamp(time::now().to_timespec())
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer
            .serialize_str(format!("{}", time::at(self.0).rfc822()).as_str())
    }
}

impl Add<time::Duration> for Timestamp {
    type Output = Timestamp;

    fn add(self, other: time::Duration) -> Timestamp {
        Timestamp(self.0.add(other))
    }
}

impl Sub<time::Duration> for Timestamp {
    type Output = Timestamp;

    fn sub(self, other: time::Duration) -> Timestamp {
        Timestamp(self.0.sub(other))
    }
}

struct TimestampVisitor;

impl<'de> Visitor<'de> for TimestampVisitor {
    type Value = Timestamp;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an rfc822 timestamp")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let datetime =
            match rfc822_sanitizer::parse_from_rfc2822_with_fallback(value) {
                Ok(dt) => dt,
                Err(e) => return Err(E::custom(format!("{}", e))),
            };

        Ok(Timestamp(time::Timespec {
            sec: datetime.timestamp(),
            nsec: 0,
        }))
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TimestampVisitor)
    }
}

impl ToSql<sql_types::Timestamp, Pg> for Timestamp {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        ToSql::<sql_types::Timestamp, Pg>::to_sql(&self.0, out)
    }
}

impl FromSql<sql_types::Timestamp, Pg> for Timestamp {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let ts = time::Timespec::from_sql(bytes)?;
        Ok(Timestamp(ts))
    }
}
