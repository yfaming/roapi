use std::sync::Arc;

use datafusion::arrow;

use crate::error::QueryError;

pub async fn exec_query(
    dfctx: &datafusion::execution::context::ExecutionContext,
    sql: &str,
) -> Result<Vec<arrow::record_batch::RecordBatch>, QueryError> {
    let plan = dfctx
        .create_logical_plan(sql)
        .map_err(QueryError::plan_sql)?;

    let df: Arc<dyn datafusion::dataframe::DataFrame> = Arc::new(
        datafusion::execution::dataframe_impl::DataFrameImpl::new(dfctx.state.clone(), &plan),
    );

    df.collect().await.map_err(QueryError::query_exec)
}

#[cfg(test)]
mod tests {
    use arrow::array::*;
    use datafusion::execution::context::ExecutionContext;

    use super::*;
    use crate::test_util::*;

    #[tokio::test]
    async fn group_by_aggregation() -> anyhow::Result<()> {
        let mut dfctx = ExecutionContext::new();
        register_table_properties(&mut dfctx)?;

        let batches = exec_query(
            &dfctx,
            r#"
            SELECT DISTINCT(Landlord), COUNT(Address)
            FROM properties
            GROUP BY Landlord
            ORDER BY Landlord
            "#,
        )
        .await?;

        let batch = &batches[0];

        assert_eq!(
            batch.column(0).as_ref(),
            Arc::new(StringArray::from(vec![
                "Carl", "Daniel", "Mike", "Roger", "Sam",
            ]))
            .as_ref(),
        );

        assert_eq!(
            batch.column(1).as_ref(),
            Arc::new(UInt64Array::from(vec![3, 3, 4, 3, 2])).as_ref(),
        );

        Ok(())
    }
}
