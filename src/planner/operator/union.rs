use crate::planner::operator::Operator;
use crate::planner::LogicalPlan;
use crate::types::tuple::SchemaRef;
use itertools::Itertools;
use std::fmt;
use std::fmt::Formatter;
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct UnionOperator {
    pub left_schema_ref: SchemaRef,
    pub right_schema_ref: SchemaRef,
}

impl UnionOperator {
    pub fn build(
        left_schema_ref: SchemaRef,
        right_schema_ref: SchemaRef,
        left_plan: LogicalPlan,
        right_plan: LogicalPlan,
    ) -> LogicalPlan {
        LogicalPlan::new(
            Operator::Union(UnionOperator {
                left_schema_ref,
                right_schema_ref,
            }),
            vec![left_plan, right_plan],
        )
    }
}

impl fmt::Display for UnionOperator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let left_columns = self
            .left_schema_ref
            .iter()
            .map(|column| column.name().to_string())
            .join(", ");
        let right_columns = self
            .right_schema_ref
            .iter()
            .map(|column| column.name().to_string())
            .join(", ");

        write!(
            f,
            "Union left: [{}], right: [{}]",
            left_columns, right_columns
        )?;

        Ok(())
    }
}
