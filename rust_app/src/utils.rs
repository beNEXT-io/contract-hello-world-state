// mod.rs

use aws_sdk_dynamodb::{error::SdkError, types::AttributeValue, types::ReturnValue, Client};
use lib::org_accordproject_helloworldstate::*;
use std::{collections::HashMap, env};

//
// Function increment_counter
//
// Finds the current `counter` value in the `state` object in DynamoDB, increments it by `1` and returns the result.
//
pub async fn increment_counter() -> Result<i64, Box<dyn std::error::Error>> {
    // Initialize the DynamoDB client.
    let config = aws_config::load_from_env().await;
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    let dynamodb_client = Client::new(&config);

    // Increment the `counter` by `1`.
    let result = dynamodb_client
        .update_item()
        .table_name(&table_name)
        .key("id", AttributeValue::S("state".to_string()))
        .update_expression("SET #c = #c + :inc")
        .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
        .expression_attribute_names("#c", "counter")
        .return_values(ReturnValue::UpdatedNew)
        .send()
        .await
        .map_err(|e| {
            println!("Error during update_item: {:?}", e);
            Box::<dyn std::error::Error>::from(format!("{:?}", e))
        })?;

    // Get the latest `counter`
    let attributes = result.attributes.ok_or("No attributes returned")?;
    let new_counter = match attributes.get("counter") {
        Some(av) => match av {
            AttributeValue::N(s) => s.parse::<i64>().unwrap_or_default(),
            _ => return Err("counter is not a number".into()),
        },
        None => return Err("counter attribute missing".into()),
    };

    println!("Successfully incremented the counter to: {}", new_counter);

    Ok(new_counter)
}

pub async fn add_data_to_database(
    hello_world_clause: &HelloWorldClause,
) -> Result<(), SdkError<aws_sdk_dynamodb::operation::put_item::PutItemError>> {
    // Initialize the DynamoDB client.
    let config = aws_config::load_from_env().await;
    let dynamodb_client = Client::new(&config);
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");

    // Add the "data" to the database.
    dynamodb_client
        .put_item()
        .table_name(&table_name)
        .item("id", AttributeValue::S("data".to_string()))
        .item(
            "_identifier",
            AttributeValue::S(hello_world_clause._identifier.clone()),
        )
        .item(
            "clause_id",
            AttributeValue::S(hello_world_clause.clause_id.clone()),
        )
        .item(
            "_class",
            AttributeValue::S(hello_world_clause._class.clone()),
        )
        .item("name", AttributeValue::S(hello_world_clause.name.clone()))
        .send()
        .await?;

    println!("Successfully saved 'data' to DynamoDB: _class: {}, clause_id: {}, _identifier: {}, name: {}", hello_world_clause._class, hello_world_clause.clause_id, hello_world_clause._identifier, hello_world_clause.name);

    Ok(())
}

pub async fn add_state_to_database(
    hello_world_clause: &HelloWorldClause,
) -> Result<(), SdkError<aws_sdk_dynamodb::operation::put_item::PutItemError>> {
    // Initialize the DynamoDB client.
    let config = aws_config::load_from_env().await;
    let dynamodb_client = Client::new(&config);
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");

    dynamodb_client
        .put_item()
        .table_name(&table_name)
        .item("id", AttributeValue::S("state".to_string()))
        .item(
            "_identifier",
            AttributeValue::S(hello_world_clause._identifier.clone()),
        )
        .item("counter", AttributeValue::N("0".to_string()))
        .item(
            "_class",
            AttributeValue::S(hello_world_clause._class.clone()),
        )
        .send()
        .await?;

    println!(
        "Successfully saved state to DynamoDB: _class: {}, _identifier: {}, counter: 0",
        hello_world_clause._class, hello_world_clause._identifier
    );

    Ok(())
}

pub async fn get_data(
    input_key: &str,
) -> Result<Option<HashMap<String, AttributeValue>>, Box<dyn std::error::Error>> {
    let config = aws_config::load_from_env().await;
    let dynamodb_client = Client::new(&config);
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");

    let result = dynamodb_client
        .get_item()
        .table_name(&table_name)
        .key("id", AttributeValue::S(input_key.to_string()))
        .send()
        .await;

    match result {
        Ok(get_item_output) => Ok(get_item_output.item),
        Err(error) => {
            println!("Error: {:?}", error);
            Err(format!("AWS SDK error: {:?}", error).into())
        }
    }
}
