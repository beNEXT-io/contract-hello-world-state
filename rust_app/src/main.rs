/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use aws_sdk_dynamodb::{types::AttributeValue, types::ReturnValue, Client};
use chrono::Utc;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use lib::org_accordproject_helloworldstate::*;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize, Debug)]
pub enum RequestType {
    MyRequest(MyRequest),
    HelloWorldClause(HelloWorldClause),
    // Add other request types here
}

#[derive(Deserialize, Serialize, Debug)]
pub enum ResponseType {
    MyResponse(MyResponse),
    HelloWorldClause(HelloWorldClause),
    // Add other response types here
}

#[derive(Deserialize, Serialize, Debug)]
struct Request {
    request: RequestType,
}

async fn increment_counter() -> Result<i64, Box<dyn std::error::Error>> {
    let config = aws_config::load_from_env().await;

    let table_name = std::env::var("TABLE_NAME").map_err(|e| {
        println!(
            "Error while retrieving TABLE_NAME environment variable: {:?}",
            e
        );
        Box::<dyn std::error::Error>::from(format!("{:?}", e))
    })?;

    let dynamodb_client = Client::new(&config);
    println!("Before update_item");

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

    println!("After update_item");

    let attributes = result.attributes.ok_or("No attributes returned")?;
    let new_counter = match attributes.get("counter") {
        Some(av) => match av {
            AttributeValue::N(s) => s.parse::<i64>().unwrap_or_default(),
            _ => return Err("counter is not a number".into()),
        },
        None => return Err("counter attribute missing".into()),
    };

    Ok(new_counter)
}

async fn handle_my_request(
    my_request: MyRequest,
) -> Result<MyResponse, Box<dyn std::error::Error>> {
    // Initialize the AWS SDK for Rust
    let config = aws_config::load_from_env().await;
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    let dynamodb_client = Client::new(&config);

    // Query the DynamoDB table for the "data" item
    let result = dynamodb_client
        .get_item()
        .table_name(table_name)
        .key("id", AttributeValue::S("data".to_string()))
        .send()
        .await;

    let counter = match increment_counter().await {
        Ok(counter) => counter,
        Err(err) => {
            println!("Failed to increment counter: {}", err);
            return Err(err);
        }
    };

    match result {
        Ok(get_item_output) => {
            // Extract the item from the response, if present
            if let Some(item) = get_item_output.item {
                if let Some(AttributeValue::S(name)) = item.get("name") {
                    return Ok(MyResponse {
                        _class: my_request._class,
                        output: format!(
                            "Hello {} - {} - counter: {}",
                            name, my_request.input, counter
                        ),
                        _timestamp: Utc::now(),
                    });
                }
            }
            Err("Item not found or 'name' field missing".into())
        }
        Err(error) => {
            println!("Error: {:?}", error);
            Err(format!("AWS SDK error: {:?}", error).into())
        }
    }
}

async fn handle_hello_world_clause(
    hello_world_clause: HelloWorldClause,
) -> Result<
    HelloWorldClause,
    aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::put_item::PutItemError>,
> {
    // Initialize the AWS SDK for Rust
    let config = aws_config::load_from_env().await;
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    let dynamodb_client = Client::new(&config);

    // First add the "data" to the database.
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

    // Second add the "state" to the database.
    // Note: In a production grade system, introduce transaction processing.
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

    Ok(HelloWorldClause {
        _class: hello_world_clause._class,
        clause_id: hello_world_clause.clause_id,
        _identifier: hello_world_clause._identifier,
        name: hello_world_clause.name,
    })
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<ResponseType, Error> {
    let response = match event.payload.request {
        RequestType::MyRequest(my_request) => match handle_my_request(my_request).await {
            Ok(my_response) => ResponseType::MyResponse(my_response),
            Err(error) => return Err(lambda_runtime::Error::from(format!("Error: {:?}", error))),
        },
        RequestType::HelloWorldClause(hello_world_clause) => {
            match handle_hello_world_clause(hello_world_clause).await {
                Ok(hello_world_clause) => ResponseType::HelloWorldClause(hello_world_clause),
                Err(error) => {
                    return Err(lambda_runtime::Error::from(format!("Error: {:?}", error)))
                }
            }
        }
    };

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
