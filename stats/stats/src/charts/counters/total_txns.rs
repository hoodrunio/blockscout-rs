use crate::{
    charts::{insert::DateValue, ChartFullUpdater},
    UpdateError,
};
use async_trait::async_trait;
use entity::sea_orm_active_enums::ChartType;
use sea_orm::{prelude::*, DbBackend, FromQueryResult, Statement};

#[derive(Default, Debug)]
pub struct TotalTxns {}

#[async_trait]
impl ChartFullUpdater for TotalTxns {
    async fn get_values(
        &self,
        blockscout: &DatabaseConnection,
    ) -> Result<Vec<DateValue>, UpdateError> {
        let data = DateValue::find_by_statement(Statement::from_string(
            DbBackend::Postgres,
            r#"
            SELECT 
                (
                    SELECT count(*)::text
                        FROM transactions
                ) AS "value",
                (
                    SELECT max(timestamp)::date as "date" 
                        FROM blocks
                        WHERE blocks.consensus = true
                ) AS "date"
            "#
            .into(),
        ))
        .one(blockscout)
        .await
        .map_err(UpdateError::BlockscoutDB)?
        .ok_or_else(|| UpdateError::Internal("query returned nothing".into()))?;

        Ok(vec![data])
    }
}

#[async_trait]
impl crate::Chart for TotalTxns {
    fn name(&self) -> &str {
        "totalTxns"
    }

    fn chart_type(&self) -> ChartType {
        ChartType::Counter
    }

    async fn update(
        &self,
        db: &DatabaseConnection,
        blockscout: &DatabaseConnection,
        full: bool,
    ) -> Result<(), UpdateError> {
        self.update_with_values(db, blockscout, full).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::simple_test::simple_test_counter;

    #[tokio::test]
    #[ignore = "needs database to run"]
    async fn update_total_txns() {
        let counter = TotalTxns::default();
        simple_test_counter("update_total_txns", counter, "6").await;
    }
}
