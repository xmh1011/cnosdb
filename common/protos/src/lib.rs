mod generated;
pub use generated::*;
pub mod models_helper;
pub mod prompb;
pub mod test_helper;

use std::fmt::{Display, Formatter};
use std::time::Duration;

use crate::kv_service::tskv_service_client::TskvServiceClient;
use crate::models::{Column, Points, Table};
use flatbuffers::{ForwardsUOffset, Vector};
use snafu::Snafu;
use tonic::transport::Channel;
use tower::timeout::Timeout;

// Default 100 MB
pub const DEFAULT_GRPC_SERVER_MESSAGE_LEN: usize = 100 * 1024 * 1024;

type PointsResult<T> = Result<T, PointsError>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum PointsError {
    #[snafu(display("{}", msg))]
    Points { msg: String },

    #[snafu(display("Flatbuffers 'Points' missing database name (db)"))]
    PointsMissingDatabaseName,

    #[snafu(display("Flatbuffers 'Points' missing tables data (tables)"))]
    PointsMissingTables,

    #[snafu(display("Flatbuffers 'Table' missing table name (tab)"))]
    TableMissingName,

    #[snafu(display("Flatbuffers 'Table' missing points data (points)"))]
    TableMissingColumns,

    #[snafu(display("Flatbuffers 'Point' missing tags data (tags)"))]
    PointMissingTags,

    #[snafu(display("Flatbuffers 'Column' missing values"))]
    ColumnMissingValues,

    #[snafu(display("Flatbuffers 'Column' missing names"))]
    ColumnMissingNames,

    #[snafu(display("Flatbuffers 'Column' missing nullbits"))]
    ColumnMissingNullbits,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldValue {
    U64(u64),
    I64(i64),
    Str(Vec<u8>),
    F64(f64),
    Bool(bool),
}

impl<'a> Points<'a> {
    pub fn db_ext(&'a self) -> PointsResult<&'a str> {
        self.db().ok_or(PointsError::PointsMissingDatabaseName)
    }

    pub fn tables_iter_ext(&'a self) -> PointsResult<impl Iterator<Item = Table<'a>>> {
        Ok(self
            .tables()
            .ok_or(PointsError::PointsMissingTables)?
            .iter())
    }
}

impl<'a> Table<'a> {
    pub fn tab_ext(&'a self) -> PointsResult<&'a str> {
        let name = self.tab().ok_or(PointsError::TableMissingName)?;
        Ok(name)
    }

    pub fn columns_iter_ext(&'a self) -> PointsResult<impl Iterator<Item = Column<'a>>> {
        Ok(self
            .columns()
            .ok_or(PointsError::TableMissingColumns)?
            .iter())
    }
}

impl<'a> Column<'a> {
    pub fn name_ext(&'a self) -> PointsResult<&'a str> {
        let name = self.name().ok_or(PointsError::ColumnMissingNames)?;
        Ok(name)
    }

    pub fn nullbit_ext(&self) -> PointsResult<Vector<u8>> {
        let nullbit = self.nullbits().ok_or(PointsError::ColumnMissingNullbits)?;
        Ok(nullbit)
    }

    pub fn string_values_len(&self) -> PointsResult<usize> {
        let len = self
            .col_values()
            .ok_or(PointsError::ColumnMissingValues)?
            .string_value()
            .map(|v| v.len())
            .unwrap_or(0);
        Ok(len)
    }

    pub fn string_values(&self) -> PointsResult<Vector<ForwardsUOffset<&str>>> {
        let values = self
            .col_values()
            .ok_or(PointsError::ColumnMissingValues)?
            .string_value()
            .unwrap_or_default();
        Ok(values)
    }

    pub fn bool_values_len(&self) -> PointsResult<usize> {
        let len = self
            .col_values()
            .ok_or(PointsError::ColumnMissingValues)?
            .bool_value()
            .map(|v| v.len())
            .unwrap_or(0);
        Ok(len)
    }

    pub fn bool_values(&self) -> PointsResult<Vector<bool>> {
        let values = self
            .col_values()
            .ok_or(PointsError::ColumnMissingValues)?
            .bool_value()
            .unwrap_or_default();
        Ok(values)
    }

    pub fn int_values_len(&self) -> PointsResult<usize> {
        let len = self
            .col_values()
            .ok_or(PointsError::ColumnMissingValues)?
            .int_value()
            .map(|v| v.len())
            .unwrap_or(0);
        Ok(len)
    }

    pub fn int_values(&self) -> PointsResult<Vector<i64>> {
        let values = self
            .col_values()
            .ok_or(PointsError::ColumnMissingValues)?
            .int_value()
            .unwrap_or_default();
        Ok(values)
    }

    pub fn float_values_len(&self) -> PointsResult<usize> {
        let len = self
            .col_values()
            .ok_or(PointsError::ColumnMissingValues)?
            .float_value()
            .map(|v| v.len())
            .unwrap_or(0);
        Ok(len)
    }

    pub fn float_values(&self) -> PointsResult<Vector<f64>> {
        let values = self
            .col_values()
            .ok_or(PointsError::ColumnMissingValues)?
            .float_value()
            .unwrap_or_default();
        Ok(values)
    }

    pub fn uint_values_len(&self) -> PointsResult<usize> {
        let len = self
            .col_values()
            .ok_or(PointsError::ColumnMissingValues)?
            .uint_value()
            .map(|v| v.len())
            .unwrap_or(0);
        Ok(len)
    }

    pub fn uint_values(&self) -> PointsResult<Vector<u64>> {
        let values = self
            .col_values()
            .ok_or(PointsError::ColumnMissingValues)?
            .uint_value()
            .unwrap_or_default();
        Ok(values)
    }
}

impl<'a> Display for Points<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "==============================")?;
        writeln!(f, "Database: {}", self.db_ext().unwrap_or("{!BAD_DB_NAME}"))?;
        writeln!(f, "------------------------------")?;
        match self.tables_iter_ext() {
            Ok(tables) => {
                for table in tables {
                    write!(
                        f,
                        "Table: {}",
                        table.tab_ext().unwrap_or("{!BAD_TABLE_NAME}")
                    )?;
                    writeln!(f, "{}", table)?;
                    writeln!(f, "------------------------------")?;
                }
            }
            Err(_) => {
                writeln!(f, "No tables")?;
            }
        }

        Ok(())
    }
}

impl<'a> Display for Table<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let columns = match self.columns_iter_ext() {
            Ok(p) => p,
            Err(_) => {
                writeln!(f, "{{!BAD_TABLE_POINTS}}")?;
                return Ok(());
            }
        };
        for column in columns {
            write!(
                f,
                "\nColumn: {}",
                column.name_ext().unwrap_or("{!BAD_COLUMN_NAME}")
            )?;
            writeln!(f, "\nColumn Type: {:?}", column.column_type())?;
            writeln!(f, "\nField Type: {:?}", column.field_type())?;
            writeln!(f, "\nColumn Value: {:?}", column.col_values())?;
            writeln!(f, "\nNullBits: {:?}", column.nullbit_ext())?;
        }
        Ok(())
    }
}

pub fn tskv_service_time_out_client(
    channel: Channel,
    time_out: Duration,
    max_message_size: usize,
) -> TskvServiceClient<Timeout<Channel>> {
    let timeout_channel = Timeout::new(channel, time_out);
    let client = TskvServiceClient::<Timeout<Channel>>::new(timeout_channel);
    let client = TskvServiceClient::max_decoding_message_size(client, max_message_size);
    TskvServiceClient::max_encoding_message_size(client, max_message_size)
}

#[cfg(test)]
pub mod test {
    use flatbuffers::FlatBufferBuilder;
    use std::collections::HashMap;

    use crate::models::{FieldType, Points};
    use crate::models_helper::create_const_points;

    #[test]
    #[ignore = "Checked by human"]
    fn test_format_fb_model_points() {
        let mut fbb = FlatBufferBuilder::new();
        let points = create_const_points(
            &mut fbb,
            "test_database",
            "test_table",
            vec![("ta", "1111"), ("tb", "22222")],
            vec![
                ("i1", 2_i64.to_be_bytes().as_slice()),
                ("f2", 2.0_f64.to_be_bytes().as_slice()),
                ("s3", "111111".as_bytes()),
            ],
            HashMap::from([
                ("i1", FieldType::Integer),
                ("f2", FieldType::Float),
                ("s3", FieldType::String),
            ]),
            0,
            10,
        );
        fbb.finish(points, None);
        let points_bytes = fbb.finished_data().to_vec();
        let fb_points = flatbuffers::root::<Points>(&points_bytes).unwrap();
        let fb_points_str = format!("{fb_points}");
        // println!("{fb_points}");
        assert_eq!(
            &fb_points_str,
            r#"==============================
Database: test_database
------------------------------
Table: test_table
Column: ta
Column Type: Tag

Field Type: String

Column Value: Some(Values { float_value: None, int_value: None, uint_value: None, bool_value: None, string_value: Some(["1111", "1111", "1111", "1111", "1111", "1111", "1111", "1111", "1111", "1111"]) })

NullBits: Ok([255, 3])

Column: tb
Column Type: Tag

Field Type: String

Column Value: Some(Values { float_value: None, int_value: None, uint_value: None, bool_value: None, string_value: Some(["22222", "22222", "22222", "22222", "22222", "22222", "22222", "22222", "22222", "22222"]) })

NullBits: Ok([255, 3])

Column: i1
Column Type: Field

Field Type: Integer

Column Value: Some(Values { float_value: None, int_value: Some([2, 2, 2, 2, 2, 2, 2, 2, 2, 2]), uint_value: None, bool_value: None, string_value: None })

NullBits: Ok([255, 3])

Column: f2
Column Type: Field

Field Type: Float

Column Value: Some(Values { float_value: Some([2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0]), int_value: None, uint_value: None, bool_value: None, string_value: None })

NullBits: Ok([255, 3])

Column: s3
Column Type: Field

Field Type: String

Column Value: Some(Values { float_value: None, int_value: None, uint_value: None, bool_value: None, string_value: Some(["111111", "111111", "111111", "111111", "111111", "111111", "111111", "111111", "111111", "111111"]) })

NullBits: Ok([255, 3])

Column: time
Column Type: Field

Field Type: Boolean

Column Value: Some(Values { float_value: None, int_value: Some([10, 10, 10, 10, 10, 10, 10, 10, 10, 10]), uint_value: None, bool_value: None, string_value: None })

NullBits: Ok([255, 3])

------------------------------
"#
        );
    }
}
