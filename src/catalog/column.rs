use crate::catalog::TableName;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::sync::Arc;

use crate::types::value::ValueRef;
use crate::types::{ColumnId, LogicalType};

pub type ColumnRef = Arc<ColumnCatalog>;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ColumnCatalog {
    pub summary: ColumnSummary,
    pub nullable: bool,
    pub desc: ColumnDesc,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ColumnSummary {
    pub id: Option<ColumnId>,
    pub name: String,
    pub table_name: Option<TableName>,
}

impl ColumnCatalog {
    pub(crate) fn new(
        column_name: String,
        nullable: bool,
        column_desc: ColumnDesc,
    ) -> ColumnCatalog {
        ColumnCatalog {
            summary: ColumnSummary {
                id: None,
                name: column_name,
                table_name: None,
            },
            nullable,
            desc: column_desc,
        }
    }

    pub(crate) fn new_dummy(column_name: String) -> ColumnCatalog {
        ColumnCatalog {
            summary: ColumnSummary {
                id: None,
                name: column_name,
                table_name: None,
            },
            nullable: true,
            desc: ColumnDesc::new(LogicalType::Varchar(None), false, false, None),
        }
    }

    pub(crate) fn summary(&self) -> &ColumnSummary {
        &self.summary
    }

    pub(crate) fn id(&self) -> Option<ColumnId> {
        self.summary.id
    }

    pub fn name(&self) -> &str {
        &self.summary.name
    }

    pub fn full_name(&self) -> String {
        if let Some(table_name) = self.table_name() {
            return format!("{}.{}", table_name, self.name());
        }
        self.name().to_string()
    }

    pub fn table_name(&self) -> Option<&TableName> {
        self.summary.table_name.as_ref()
    }

    pub fn set_table_name(&mut self, table_name: TableName) {
        self.summary.table_name = Some(table_name);
    }

    pub fn datatype(&self) -> &LogicalType {
        &self.desc.column_datatype
    }

    pub(crate) fn default_value(&self) -> Option<ValueRef> {
        self.desc.default.clone()
    }

    #[allow(dead_code)]
    pub(crate) fn desc(&self) -> &ColumnDesc {
        &self.desc
    }
}

/// The descriptor of a column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ColumnDesc {
    pub(crate) column_datatype: LogicalType,
    pub(crate) is_primary: bool,
    pub(crate) is_unique: bool,
    pub(crate) default: Option<ValueRef>,
}

impl ColumnDesc {
    pub(crate) const fn new(
        column_datatype: LogicalType,
        is_primary: bool,
        is_unique: bool,
        default: Option<ValueRef>,
    ) -> ColumnDesc {
        ColumnDesc {
            column_datatype,
            is_primary,
            is_unique,
            default,
        }
    }

    pub(crate) fn is_index(&self) -> bool {
        self.is_unique || self.is_primary
    }
}
