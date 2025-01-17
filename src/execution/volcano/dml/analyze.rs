use crate::catalog::{ColumnCatalog, ColumnRef, TableMeta, TableName};
use crate::errors::DatabaseError;
use crate::execution::volcano::{build_read, BoxedExecutor, WriteExecutor};
use crate::optimizer::core::column_meta::ColumnMeta;
use crate::optimizer::core::histogram::HistogramBuilder;
use crate::planner::operator::analyze::AnalyzeOperator;
use crate::planner::LogicalPlan;
use crate::storage::Transaction;
use crate::types::tuple::Tuple;
use crate::types::value::DataValue;
use futures_async_stream::try_stream;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fmt, fs};

const DEFAULT_NUM_OF_BUCKETS: usize = 100;
const DEFAULT_COLUMN_METAS_PATH: &str = "fnck_sql_column_metas";

pub struct Analyze {
    table_name: TableName,
    input: LogicalPlan,
    columns: Vec<ColumnRef>,
}

impl From<(AnalyzeOperator, LogicalPlan)> for Analyze {
    fn from(
        (
            AnalyzeOperator {
                table_name,
                columns,
            },
            input,
        ): (AnalyzeOperator, LogicalPlan),
    ) -> Self {
        Analyze {
            table_name,
            input,
            columns,
        }
    }
}

impl<T: Transaction> WriteExecutor<T> for Analyze {
    fn execute_mut(self, transaction: &mut T) -> BoxedExecutor {
        self._execute(transaction)
    }
}

impl Analyze {
    #[try_stream(boxed, ok = Tuple, error = DatabaseError)]
    pub async fn _execute<T: Transaction>(self, transaction: &mut T) {
        let Analyze {
            table_name,
            input,
            columns,
        } = self;

        let mut builders = HashMap::with_capacity(columns.len());

        for column in &columns {
            builders.insert(column.id(), HistogramBuilder::new(column, None)?);
        }

        #[for_await]
        for tuple in build_read(input, transaction) {
            let Tuple {
                schema_ref, values, ..
            } = tuple?;

            for (i, column) in schema_ref.iter().enumerate() {
                if !column.desc.is_index() {
                    continue;
                }

                if let Some(builder) = builders.get_mut(&column.id()) {
                    builder.append(&values[i])?
                }
            }
        }
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("It's the end of the world!")
            .as_secs();
        let dir_path = dirs::config_dir()
            .expect("Your system does not have a Config directory!")
            .join(DEFAULT_COLUMN_METAS_PATH)
            .join(table_name.as_str())
            .join(ts.to_string());
        fs::create_dir_all(&dir_path)?;

        let mut meta = TableMeta::empty(table_name.clone());

        for (column_id, builder) in builders {
            let path = dir_path.join(column_id.unwrap().to_string());
            let (histogram, sketch) = builder.build(DEFAULT_NUM_OF_BUCKETS)?;

            ColumnMeta::new(histogram, sketch).to_file(&path)?;

            meta.colum_meta_paths.push(path.to_string_lossy().into());
        }
        transaction.save_table_meta(&meta)?;

        let columns: Vec<ColumnRef> = vec![Arc::new(ColumnCatalog::new_dummy(
            "COLUMN_META_PATH".to_string(),
        ))];
        let values = meta
            .colum_meta_paths
            .into_iter()
            .map(|path| Arc::new(DataValue::Utf8(Some(path))))
            .collect_vec();

        yield Tuple {
            id: None,
            schema_ref: Arc::new(columns),
            values,
        };
    }
}

impl fmt::Display for AnalyzeOperator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let columns = self
            .columns
            .iter()
            .map(|column| column.name().to_string())
            .join(", ");

        write!(f, "Analyze {} -> [{}]", self.table_name, columns)?;

        Ok(())
    }
}
